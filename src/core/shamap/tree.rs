//! ShaMap: a HAMT-optimized radix-16 Merkle trie for the XRP Ledger.
//!
//! Optimizations over the naive implementation:
//!
//! - **HAMT bitmap indexing**: Inner nodes use a 16-bit bitmap and compact `Vec`
//!   to store only non-empty children. Sparse nodes (common in deep trie levels)
//!   use ~70-80% less memory than a full 16-slot array. Branch lookup is O(1) via
//!   `popcount`. See: Bagwell (2001), "Ideal Hash Trees".
//!
//! - **Hash caching with dirty-flag propagation**: Each node caches its hash in a
//!   `Cell<Option<Hash256>>`. Repeated `hash()` calls are O(1). Mutations
//!   (`add_item`, `remove_item`) invalidate only the path from the modified leaf
//!   to the root — O(log₁₆ n) invalidations, not O(n) recomputation.
//!
//! - **Single-buffer inner hash**: The 516-byte input (4-byte prefix + 16 × 32-byte
//!   child hashes) is assembled in a zero-initialized stack buffer and hashed in one
//!   `sha512half` call. Empty branches contribute `ZERO_256` for free (from the
//!   zero-init), and only populated branches are copied.
//!
//! - **In-place mutation**: Recursing into an existing inner node mutates it
//!   directly via `&mut` without allocating new nodes.

use alloc::vec::Vec;
use core::cell::Cell;

use super::hash_prefix;
use super::sha512half::sha512half;
use super::sha512half::Sha512Half;

/// A 256-bit ShaMap key (index).
pub type ShaMapIndex = [u8; 32];

/// A 256-bit hash value.
pub type Hash256 = [u8; 32];

/// The zero hash, used for empty branches.
pub const ZERO_256: Hash256 = [0u8; 32];

/// Size of the inner-node hash input: 4-byte prefix + 16 × 32-byte child hashes.
const INNER_HASH_INPUT_LEN: usize = 4 + 16 * 32;

// ---------------------------------------------------------------------------
// Nibble extraction (optimized with bitwise ops)
// ---------------------------------------------------------------------------

/// Extract the nibble (0..15) at the given depth from a 256-bit index.
///
/// Depth 0 = high nibble of byte 0, depth 1 = low nibble of byte 0, etc.
/// Max depth for a 32-byte key is 63.
#[inline(always)]
fn nibble(index: &ShaMapIndex, depth: usize) -> usize {
    let byte = index[depth >> 1];
    if depth & 1 == 0 {
        (byte >> 4) as usize
    } else {
        (byte & 0x0F) as usize
    }
}

// ---------------------------------------------------------------------------
// HAMT bitmap helpers
// ---------------------------------------------------------------------------

/// Returns true if branch `n` (0..15) is present in the bitmap.
#[inline(always)]
fn has_branch(bitmap: u16, n: usize) -> bool {
    bitmap & (1u16 << n) != 0
}

/// Compact index for branch `n`: the number of set bits below position `n`.
///
/// Works correctly whether or not bit `n` is currently set.
#[inline(always)]
fn compact_index(bitmap: u16, n: usize) -> usize {
    (bitmap & ((1u16 << n).wrapping_sub(1))).count_ones() as usize
}

// ---------------------------------------------------------------------------
// Leaf node
// ---------------------------------------------------------------------------

/// A leaf node in the ShaMap.
pub struct ShaMapLeaf {
    /// The 256-bit index (key) for this leaf.
    pub index: ShaMapIndex,
    /// The 4-byte hash prefix for domain separation.
    pub hash_prefix: [u8; 4],
    /// The serialized item data.
    pub data: Vec<u8>,
    /// Whether to include the index in the leaf hash.
    ///
    /// - `true` (default): hash = `sha512half(prefix || data || index)`.
    ///   Used for `ACCOUNT_STATE` and `TRANSACTION_METADATA` node types.
    /// - `false`: hash = `sha512half(prefix || data)`.
    ///   Used for `TRANSACTION_NO_METADATA` node type (the index IS the tx hash).
    include_index_in_hash: bool,
    /// Cached leaf hash. Computed once on first access.
    hash_cache: Cell<Option<Hash256>>,
}

impl ShaMapLeaf {
    /// Create a new leaf node (default: index included in hash).
    pub fn new(index: ShaMapIndex, hash_prefix: [u8; 4], data: Vec<u8>) -> Self {
        ShaMapLeaf {
            index,
            hash_prefix,
            data,
            include_index_in_hash: true,
            hash_cache: Cell::new(None),
        }
    }

    /// Create a leaf where the hash does NOT include the index.
    ///
    /// Used for `TRANSACTION_NO_METADATA` nodes in xrpl.js, where the
    /// leaf hash is simply `sha512half(TRANSACTION_ID || data)`.
    pub fn new_no_index(index: ShaMapIndex, hash_prefix: [u8; 4], data: Vec<u8>) -> Self {
        ShaMapLeaf {
            index,
            hash_prefix,
            data,
            include_index_in_hash: false,
            hash_cache: Cell::new(None),
        }
    }

    /// Compute (or return cached) leaf hash.
    pub fn hash(&self) -> Hash256 {
        if let Some(h) = self.hash_cache.get() {
            return h;
        }
        let mut hasher = Sha512Half::new();
        hasher.update(&self.hash_prefix);
        hasher.update(&self.data);
        if self.include_index_in_hash {
            hasher.update(&self.index);
        }
        let h = hasher.finish();
        self.hash_cache.set(Some(h));
        h
    }
}

// ---------------------------------------------------------------------------
// Inner node (HAMT bitmap-indexed)
// ---------------------------------------------------------------------------

/// A HAMT-style inner node with bitmap-indexed children.
///
/// Instead of a fixed 16-slot array, stores a `u16` bitmap indicating which
/// branches are populated and a compact `Vec` of only those children. The
/// index into the Vec for branch `n` is `popcount(bitmap & ((1 << n) - 1))`.
pub struct ShaMapInner {
    depth: u8,
    bitmap: u16,
    children: Vec<ShaMapNode>,
    hash_cache: Cell<Option<Hash256>>,
}

/// A node in the ShaMap: either a leaf or an inner node.
pub enum ShaMapNode {
    Leaf(ShaMapLeaf),
    Inner(ShaMapInner),
}

impl ShaMapNode {
    /// Compute (or return cached) hash of this node.
    fn hash(&self) -> Hash256 {
        match self {
            ShaMapNode::Leaf(leaf) => leaf.hash(),
            ShaMapNode::Inner(inner) => inner.hash(),
        }
    }

    fn is_leaf(&self) -> bool {
        matches!(self, ShaMapNode::Leaf(_))
    }

    fn is_inner(&self) -> bool {
        matches!(self, ShaMapNode::Inner(_))
    }

    fn as_inner_mut(&mut self) -> &mut ShaMapInner {
        match self {
            ShaMapNode::Inner(inner) => inner,
            _ => panic!("expected inner node"),
        }
    }
}

impl ShaMapInner {
    /// Create a new empty inner node at the given depth.
    ///
    /// Crate-private: `depth` must satisfy `depth < 64` for all subsequent
    /// operations on the node to be safe. Constructing an inner node with
    /// `depth >= 64` would cause `nibble()` to index out of bounds on the
    /// first call to `add_item` / `remove_item` / `contains` / `get`. Only
    /// `ShaMap::new` and internal tree operations should call this.
    pub(crate) fn new(depth: u8) -> Self {
        ShaMapInner {
            depth,
            bitmap: 0,
            children: Vec::new(),
            hash_cache: Cell::new(None),
        }
    }

    /// Returns true if all branches are empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.bitmap == 0
    }

    /// Number of populated branches.
    #[inline]
    pub fn branch_count(&self) -> u32 {
        self.bitmap.count_ones()
    }

    /// Invalidate the cached hash (called on any mutation).
    #[inline]
    fn invalidate(&self) {
        self.hash_cache.set(None);
    }

    /// Get a reference to the child at branch `n` (0..15).
    pub fn get_child(&self, n: usize) -> Option<&ShaMapNode> {
        if !has_branch(self.bitmap, n) {
            return None;
        }
        let idx = compact_index(self.bitmap, n);
        Some(&self.children[idx])
    }

    /// Compute (or return cached) inner node hash.
    ///
    /// Uses a zero-initialized 516-byte stack buffer. Only populated branches
    /// are copied in (via bitmap iteration), so empty branches contribute
    /// `ZERO_256` for free. The entire buffer is hashed in one `sha512half` call.
    pub fn hash(&self) -> Hash256 {
        if let Some(h) = self.hash_cache.get() {
            return h;
        }
        if self.bitmap == 0 {
            self.hash_cache.set(Some(ZERO_256));
            return ZERO_256;
        }

        let mut buf = [0u8; INNER_HASH_INPUT_LEN];
        buf[..4].copy_from_slice(&hash_prefix::INNER_NODE);

        // Iterate only over set bits in the bitmap
        let mut bits = self.bitmap;
        let mut child_idx = 0;
        while bits != 0 {
            let branch = bits.trailing_zeros() as usize;
            let offset = 4 + branch * 32;
            let child_hash = self.children[child_idx].hash();
            buf[offset..offset + 32].copy_from_slice(&child_hash);
            child_idx += 1;
            bits &= bits - 1; // clear lowest set bit
        }

        let h = sha512half(&buf);
        self.hash_cache.set(Some(h));
        h
    }

    /// Insert a leaf, creating sub-trees on collision.
    ///
    /// Returns `true` if a new leaf was inserted, `false` if an existing leaf
    /// with the same key was replaced.
    ///
    /// Hash caches are invalidated along the insertion path only.
    pub fn add_item(&mut self, leaf: ShaMapLeaf) -> bool {
        let slot = nibble(&leaf.index, self.depth as usize);
        self.invalidate();

        if !has_branch(self.bitmap, slot) {
            // Empty slot: insert leaf directly
            let idx = compact_index(self.bitmap, slot);
            self.bitmap |= 1u16 << slot;
            self.children.insert(idx, ShaMapNode::Leaf(leaf));
            return true;
        }

        let idx = compact_index(self.bitmap, slot);

        // In-place mutation: avoid unboxing + re-boxing for inner node recursion
        if self.children[idx].is_inner() {
            return self.children[idx].as_inner_mut().add_item(leaf);
        }

        // Leaf collision: check for duplicate key first
        if let ShaMapNode::Leaf(ref existing_leaf) = self.children[idx] {
            if existing_leaf.index == leaf.index {
                // Duplicate key: replace the existing leaf
                self.children[idx] = ShaMapNode::Leaf(leaf);
                return false;
            }
        }

        // Different keys in the same slot: replace with a new inner node containing both leaves
        let existing = self.children.remove(idx);
        self.bitmap &= !(1u16 << slot);

        let mut new_inner =
            ShaMapInner::new(self.depth.checked_add(1).expect("ShaMap depth overflow"));
        match existing {
            ShaMapNode::Leaf(existing_leaf) => new_inner.add_item(existing_leaf),
            ShaMapNode::Inner(_) => unreachable!(),
        };
        new_inner.add_item(leaf);

        let new_idx = compact_index(self.bitmap, slot);
        self.bitmap |= 1u16 << slot;
        self.children.insert(new_idx, ShaMapNode::Inner(new_inner));
        true
    }

    /// Remove the item at `index`, collapsing single-child inner nodes.
    ///
    /// Returns `true` if the item was found and removed.
    pub fn remove_item(&mut self, index: &ShaMapIndex) -> bool {
        let slot = nibble(index, self.depth as usize);

        if !has_branch(self.bitmap, slot) {
            return false;
        }

        let idx = compact_index(self.bitmap, slot);

        match &self.children[idx] {
            ShaMapNode::Leaf(leaf) => {
                if leaf.index != *index {
                    return false;
                }
                // Remove the leaf
                self.children.remove(idx);
                self.bitmap &= !(1u16 << slot);
                self.invalidate();
                true
            }
            ShaMapNode::Inner(_) => {
                // Recurse into the inner node
                let removed = self.children[idx].as_inner_mut().remove_item(index);

                if !removed {
                    return false;
                }

                self.invalidate();

                // Collapse: if the inner node now has exactly one child that is
                // a leaf, replace the inner node with that leaf
                let should_collapse = {
                    let inner = match &self.children[idx] {
                        ShaMapNode::Inner(i) => i,
                        _ => unreachable!(),
                    };
                    inner.branch_count() == 1 && {
                        let only_branch = inner.bitmap.trailing_zeros() as usize;
                        inner.get_child(only_branch).unwrap().is_leaf()
                    }
                };

                if should_collapse {
                    let inner_node = self.children.remove(idx);
                    self.bitmap &= !(1u16 << slot);

                    let inner = match inner_node {
                        ShaMapNode::Inner(i) => i,
                        _ => unreachable!(),
                    };

                    let only_branch = inner.bitmap.trailing_zeros() as usize;
                    let only_idx = compact_index(inner.bitmap, only_branch);
                    let leaf_node = inner.children.into_iter().nth(only_idx).unwrap();

                    let new_idx = compact_index(self.bitmap, slot);
                    self.bitmap |= 1u16 << slot;
                    self.children.insert(new_idx, leaf_node);
                }

                true
            }
        }
    }

    /// Check if an item with the given index exists in this subtree.
    pub fn contains(&self, index: &ShaMapIndex) -> bool {
        let slot = nibble(index, self.depth as usize);

        if !has_branch(self.bitmap, slot) {
            return false;
        }

        let idx = compact_index(self.bitmap, slot);
        match &self.children[idx] {
            ShaMapNode::Leaf(leaf) => leaf.index == *index,
            ShaMapNode::Inner(inner) => inner.contains(index),
        }
    }

    /// Get a reference to the leaf at `index`, if it exists.
    pub fn get(&self, index: &ShaMapIndex) -> Option<&ShaMapLeaf> {
        let slot = nibble(index, self.depth as usize);

        if !has_branch(self.bitmap, slot) {
            return None;
        }

        let idx = compact_index(self.bitmap, slot);
        match &self.children[idx] {
            ShaMapNode::Leaf(leaf) => {
                if leaf.index == *index {
                    Some(leaf)
                } else {
                    None
                }
            }
            ShaMapNode::Inner(inner) => inner.get(index),
        }
    }

    /// Count the total number of leaf items in this subtree.
    pub fn len(&self) -> usize {
        let mut count = 0;
        for child in &self.children {
            match child {
                ShaMapNode::Leaf(_) => count += 1,
                ShaMapNode::Inner(inner) => count += inner.len(),
            }
        }
        count
    }
}

// ---------------------------------------------------------------------------
// Inclusion proofs
// ---------------------------------------------------------------------------

/// A level in a ShaMap inclusion proof.
pub struct ProofLevel {
    /// Which branch (0..15) was taken at this level.
    pub nibble: u8,
    /// Hashes of all 16 branches at this inner node. The branch at `nibble`
    /// is replaced by the hash propagated from below during verification.
    pub sibling_hashes: [Hash256; 16],
}

/// An inclusion proof for a single leaf in the ShaMap.
///
/// Fields are `pub` for interoperability (e.g. serialization, fuzz harnesses),
/// but callers SHOULD obtain proofs via [`ShaMap::extract_proof`] rather than
/// constructing `ShaMapProof` manually. Directly constructed proofs will only
/// verify if every `level.nibble` in `path` matches the nibble of `index` at
/// the corresponding depth: `verify_proof` enforces this binding to prevent
/// an attacker from routing a valid `leaf_hash` through an arbitrary path.
pub struct ShaMapProof {
    /// The index of the leaf being proved.
    pub index: ShaMapIndex,
    /// The hash of the leaf.
    pub leaf_hash: Hash256,
    /// The proof path from root to leaf.
    pub path: Vec<ProofLevel>,
}

/// Verify a ShaMap inclusion proof against an expected root hash.
///
/// Walks the path from leaf to root, recomputing inner hashes at each level,
/// using the single-buffer technique for consistency with tree hashing.
///
/// The verifier also binds each level's `nibble` to `proof.index`: at depth `d`
/// (counted from the root, matching the `path` order), the nibble MUST equal
/// `nibble(&proof.index, d)`. Without this check, `TRANSACTION_NO_METADATA`
/// leaves (whose hash excludes the index) could be routed through an attacker
/// chosen path to a different position in the tree.
pub fn verify_proof(proof: &ShaMapProof, expected_root: &Hash256) -> bool {
    // Reject absurd paths early: a 256-bit index has at most 64 nibbles, so
    // any path longer than that cannot correspond to a well-formed tree.
    if proof.path.len() > 64 {
        return false;
    }

    // Bind each level's nibble to the proof index at the corresponding depth.
    // `path` is in root-to-leaf order: path[0] is depth 0, path[1] is depth 1, ...
    for (depth, level) in proof.path.iter().enumerate() {
        if level.nibble > 15 {
            return false;
        }
        if level.nibble as usize != nibble(&proof.index, depth) {
            return false;
        }
    }

    let mut current_hash = proof.leaf_hash;

    for level in proof.path.iter().rev() {
        let mut buf = [0u8; INNER_HASH_INPUT_LEN];
        buf[..4].copy_from_slice(&hash_prefix::INNER_NODE);

        for i in 0..16 {
            let offset = 4 + i * 32;
            if i == level.nibble as usize {
                buf[offset..offset + 32].copy_from_slice(&current_hash);
            } else {
                buf[offset..offset + 32].copy_from_slice(&level.sibling_hashes[i]);
            }
        }

        current_hash = sha512half(&buf);
    }

    current_hash == *expected_root
}

// ---------------------------------------------------------------------------
// Top-level ShaMap
// ---------------------------------------------------------------------------

/// The top-level ShaMap structure.
///
/// Wraps a root `ShaMapInner` node at depth 0.
pub struct ShaMap {
    root: ShaMapInner,
}

impl ShaMap {
    /// Create a new empty ShaMap.
    pub fn new() -> Self {
        ShaMap {
            root: ShaMapInner::new(0),
        }
    }

    /// Add an item to the ShaMap (leaf hash includes the index).
    ///
    /// Returns `true` if a new item was inserted, `false` if an existing item
    /// with the same index was replaced.
    pub fn add_item(&mut self, index: ShaMapIndex, hash_prefix: [u8; 4], data: Vec<u8>) -> bool {
        self.root
            .add_item(ShaMapLeaf::new(index, hash_prefix, data))
    }

    /// Add an item whose leaf hash does NOT include the index.
    ///
    /// Used for `TRANSACTION_NO_METADATA` nodes where the hash is
    /// `sha512half(TRANSACTION_ID || data)`.
    ///
    /// Returns `true` if a new item was inserted, `false` if an existing item
    /// with the same index was replaced.
    pub fn add_item_no_index(
        &mut self,
        index: ShaMapIndex,
        hash_prefix: [u8; 4],
        data: Vec<u8>,
    ) -> bool {
        self.root
            .add_item(ShaMapLeaf::new_no_index(index, hash_prefix, data))
    }

    /// Remove the item at `index`. Returns `true` if found and removed.
    pub fn remove_item(&mut self, index: &ShaMapIndex) -> bool {
        self.root.remove_item(index)
    }

    /// Check if an item with the given index exists.
    pub fn contains(&self, index: &ShaMapIndex) -> bool {
        self.root.contains(index)
    }

    /// Get a reference to the leaf at `index`.
    pub fn get(&self, index: &ShaMapIndex) -> Option<&ShaMapLeaf> {
        self.root.get(index)
    }

    /// Number of items in the ShaMap.
    pub fn len(&self) -> usize {
        self.root.len()
    }

    /// Returns true if the ShaMap is empty.
    pub fn is_empty(&self) -> bool {
        self.root.is_empty()
    }

    /// Compute the root hash of the ShaMap.
    pub fn hash(&self) -> Hash256 {
        self.root.hash()
    }

    /// Extract an inclusion proof for the item at `target`.
    ///
    /// Returns `None` if the target is not in the tree.
    pub fn extract_proof(&self, target: &ShaMapIndex) -> Option<ShaMapProof> {
        let mut path = Vec::new();
        let leaf_hash = Self::collect_proof(&self.root, target, &mut path)?;

        Some(ShaMapProof {
            index: *target,
            leaf_hash,
            path,
        })
    }

    /// Recursively walk the tree collecting proof levels (root-to-leaf order).
    fn collect_proof(
        inner: &ShaMapInner,
        target: &ShaMapIndex,
        path: &mut Vec<ProofLevel>,
    ) -> Option<Hash256> {
        let slot = nibble(target, inner.depth as usize);

        // Collect all 16 sibling hashes at this level
        let mut sibling_hashes = [ZERO_256; 16];
        let mut bits = inner.bitmap;
        let mut child_idx = 0;
        while bits != 0 {
            let branch = bits.trailing_zeros() as usize;
            sibling_hashes[branch] = inner.children[child_idx].hash();
            child_idx += 1;
            bits &= bits - 1;
        }

        path.push(ProofLevel {
            nibble: slot as u8,
            sibling_hashes,
        });

        if !has_branch(inner.bitmap, slot) {
            path.pop();
            return None;
        }

        let idx = compact_index(inner.bitmap, slot);
        match &inner.children[idx] {
            ShaMapNode::Leaf(leaf) => {
                if leaf.index == *target {
                    Some(leaf.hash())
                } else {
                    path.pop();
                    None
                }
            }
            ShaMapNode::Inner(child_inner) => {
                let result = Self::collect_proof(child_inner, target, path);
                if result.is_none() {
                    path.pop();
                }
                result
            }
        }
    }
}

impl Default for ShaMap {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    // --- Nibble extraction ---

    #[test]
    fn test_nibble_extraction() {
        let mut idx = [0u8; 32];
        idx[0] = 0xAB;
        idx[1] = 0xCD;

        assert_eq!(nibble(&idx, 0), 0xA);
        assert_eq!(nibble(&idx, 1), 0xB);
        assert_eq!(nibble(&idx, 2), 0xC);
        assert_eq!(nibble(&idx, 3), 0xD);
    }

    #[test]
    fn test_nibble_all_zeros() {
        let idx = [0u8; 32];
        for depth in 0..64 {
            assert_eq!(nibble(&idx, depth), 0);
        }
    }

    #[test]
    fn test_nibble_all_ones() {
        let idx = [0xFF; 32];
        for depth in 0..64 {
            assert_eq!(nibble(&idx, depth), 0xF);
        }
    }

    // --- HAMT bitmap helpers ---

    #[test]
    fn test_has_branch() {
        let bitmap: u16 = 0b0000_0000_0010_0101; // branches 0, 2, 5
        assert!(has_branch(bitmap, 0));
        assert!(!has_branch(bitmap, 1));
        assert!(has_branch(bitmap, 2));
        assert!(!has_branch(bitmap, 3));
        assert!(!has_branch(bitmap, 4));
        assert!(has_branch(bitmap, 5));
    }

    #[test]
    fn test_compact_index() {
        let bitmap: u16 = 0b0000_0000_0010_0101; // branches 0, 2, 5
        assert_eq!(compact_index(bitmap, 0), 0); // branch 0 -> index 0
        assert_eq!(compact_index(bitmap, 2), 1); // branch 2 -> index 1
        assert_eq!(compact_index(bitmap, 5), 2); // branch 5 -> index 2
    }

    #[test]
    fn test_compact_index_insertion_point() {
        // compact_index also works for unset bits (gives insertion point)
        let bitmap: u16 = 0b0000_0000_0010_0101; // branches 0, 2, 5
        assert_eq!(compact_index(bitmap, 1), 1); // insert before branch 2
        assert_eq!(compact_index(bitmap, 3), 2); // insert after branch 2
        assert_eq!(compact_index(bitmap, 4), 2); // same position as 3
    }

    // --- Empty / basic tree ---

    #[test]
    fn test_empty_shamap_is_zero() {
        let map = ShaMap::new();
        assert_eq!(map.hash(), ZERO_256);
        assert!(map.is_empty());
        assert_eq!(map.len(), 0);
    }

    #[test]
    fn test_default_shamap() {
        let map = ShaMap::default();
        assert_eq!(map.hash(), ZERO_256);
    }

    #[test]
    fn test_empty_inner_hash_is_zero() {
        let inner = ShaMapInner::new(0);
        assert!(inner.is_empty());
        assert_eq!(inner.hash(), ZERO_256);
    }

    // --- Single item ---

    #[test]
    fn test_single_item_hash() {
        let mut map = ShaMap::new();
        let index = [0u8; 32];
        let data = vec![1, 2, 3, 4];
        let prefix = [0x01, 0x03, 0x03, 0x07];

        map.add_item(index, prefix, data.clone());
        assert_eq!(map.len(), 1);
        assert!(!map.is_empty());

        // Manually compute expected leaf hash
        let leaf = ShaMapLeaf::new(index, prefix, data);
        let leaf_hash = leaf.hash();

        // Root inner: INNER_NODE prefix + branch 0 = leaf_hash, rest = ZERO_256
        let mut h = Sha512Half::new();
        h.update(&hash_prefix::INNER_NODE);
        for i in 0..16 {
            if i == 0 {
                h.update(&leaf_hash);
            } else {
                h.update(&ZERO_256);
            }
        }
        let expected_root = h.finish();

        assert_eq!(map.hash(), expected_root);
    }

    // --- Two items ---

    #[test]
    fn test_two_items_different_nibbles() {
        let mut map = ShaMap::new();

        let mut idx1 = [0u8; 32];
        idx1[0] = 0x00;
        let mut idx2 = [0u8; 32];
        idx2[0] = 0x10;

        let prefix = [0x01, 0x03, 0x03, 0x07];
        map.add_item(idx1, prefix, vec![1]);
        map.add_item(idx2, prefix, vec![2]);

        assert_eq!(map.len(), 2);
        assert_ne!(map.hash(), ZERO_256);
    }

    #[test]
    fn test_two_items_same_first_nibble_creates_inner() {
        let mut map = ShaMap::new();

        let mut idx1 = [0u8; 32];
        idx1[0] = 0xA0;
        let mut idx2 = [0u8; 32];
        idx2[0] = 0xA1;

        let prefix = [0x01, 0x03, 0x03, 0x07];
        map.add_item(idx1, prefix, vec![1]);
        map.add_item(idx2, prefix, vec![2]);

        // Root branch 0xA should be an inner node at depth 1
        match map.root.get_child(0xA) {
            Some(ShaMapNode::Inner(inner)) => {
                assert_eq!(inner.depth, 1);
                assert!(inner.get_child(0).is_some());
                assert!(inner.get_child(1).is_some());
            }
            Some(ShaMapNode::Leaf(_)) => panic!("expected inner node, got leaf"),
            None => panic!("expected branch 0xA to be occupied"),
        }
    }

    // --- Order independence ---

    #[test]
    fn test_order_independence() {
        let prefix = [0x01, 0x03, 0x03, 0x07];
        let hex_items = [
            "0000000000000000000000000000000000000000000000000000000000000000",
            "1000000000000000000000000000000000000000000000000000000000000000",
            "1100000000000000000000000000000000000000000000000000000000000000",
            "7000DE445E22CB9BB7E1717589FA858736BAA5FD192310E20000000000000000",
            "7000DE445E22CB9BB7E1717589FA858736BAA5FD192310E21000000000000000",
            "7000DE445E22CB9BB7E1717589FA858736BAA5FD192310E22000000000000000",
            "7000DE445E22CB9BB7E1717589FA858736BAA5FD192310E23000000000000000",
            "1200000000000000000000000000000000000000000000000000000000000000",
            "1220000000000000000000000000000000000000000000000000000000000000",
        ];

        let items: Vec<[u8; 32]> = hex_items
            .iter()
            .map(|h| {
                let bytes = hex::decode(h).unwrap();
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&bytes);
                arr
            })
            .collect();

        let mut map1 = ShaMap::new();
        for item in &items {
            map1.add_item(*item, prefix, item.to_vec());
        }
        let hash1 = map1.hash();

        let mut map2 = ShaMap::new();
        for item in items.iter().rev() {
            map2.add_item(*item, prefix, item.to_vec());
        }
        let hash2 = map2.hash();

        assert_eq!(hash1, hash2, "hash must be order-independent");
        assert_ne!(hash1, ZERO_256);
    }

    // --- Leaf hash correctness ---

    #[test]
    fn test_leaf_hash_correctness() {
        let index = [0xAB; 32];
        let prefix = [0x53, 0x4E, 0x44, 0x00];
        let data = vec![0x01, 0x02, 0x03];

        let leaf = ShaMapLeaf::new(index, prefix, data.clone());

        let mut h = Sha512Half::new();
        h.update(&prefix);
        h.update(&data);
        h.update(&index);
        let expected = h.finish();

        assert_eq!(leaf.hash(), expected);
    }

    // --- Depth verification ---

    #[test]
    fn test_depth_verification() {
        let prefix = [0x01, 0x03, 0x03, 0x07];
        let mut map = ShaMap::new();

        // Items sharing first 3 nibbles (0xABC) but differing at 4th
        let mut idx1 = [0u8; 32];
        idx1[0] = 0xAB;
        idx1[1] = 0xC0;

        let mut idx2 = [0u8; 32];
        idx2[0] = 0xAB;
        idx2[1] = 0xC1;

        map.add_item(idx1, prefix, vec![1]);
        map.add_item(idx2, prefix, vec![2]);

        // Navigate: root -> A -> B -> C -> two leaves
        let branch_a = match map.root.get_child(0xA) {
            Some(ShaMapNode::Inner(inner)) => inner,
            _ => panic!("expected inner at depth 1"),
        };
        assert_eq!(branch_a.depth, 1);

        let branch_b = match branch_a.get_child(0xB) {
            Some(ShaMapNode::Inner(inner)) => inner,
            _ => panic!("expected inner at depth 2"),
        };
        assert_eq!(branch_b.depth, 2);

        let branch_c = match branch_b.get_child(0xC) {
            Some(ShaMapNode::Inner(inner)) => inner,
            _ => panic!("expected inner at depth 3"),
        };
        assert_eq!(branch_c.depth, 3);

        assert!(matches!(branch_c.get_child(0), Some(ShaMapNode::Leaf(_))));
        assert!(matches!(branch_c.get_child(1), Some(ShaMapNode::Leaf(_))));
    }

    // --- Hash caching ---

    #[test]
    fn test_hash_caching_returns_same_value() {
        let mut map = ShaMap::new();
        let prefix = [0x01, 0x03, 0x03, 0x07];

        for i in 0u8..10 {
            let mut idx = [0u8; 32];
            idx[0] = i * 0x18;
            map.add_item(idx, prefix, vec![i]);
        }

        let hash1 = map.hash();
        let hash2 = map.hash();
        assert_eq!(hash1, hash2, "cached hash must be identical");
    }

    #[test]
    fn test_hash_invalidation_on_add() {
        let mut map = ShaMap::new();
        let prefix = [0x01, 0x03, 0x03, 0x07];

        let mut idx1 = [0u8; 32];
        idx1[0] = 0xAA;
        map.add_item(idx1, prefix, vec![1]);
        let hash1 = map.hash();

        let mut idx2 = [0u8; 32];
        idx2[0] = 0xBB;
        map.add_item(idx2, prefix, vec![2]);
        let hash2 = map.hash();

        assert_ne!(hash1, hash2, "hash must change after adding an item");
    }

    #[test]
    fn test_hash_invalidation_on_remove() {
        let mut map = ShaMap::new();
        let prefix = [0x01, 0x03, 0x03, 0x07];

        let mut idx1 = [0u8; 32];
        idx1[0] = 0xAA;
        let mut idx2 = [0u8; 32];
        idx2[0] = 0xBB;

        map.add_item(idx1, prefix, vec![1]);
        map.add_item(idx2, prefix, vec![2]);
        let hash_two = map.hash();

        map.remove_item(&idx2);
        let hash_one = map.hash();

        assert_ne!(hash_two, hash_one, "hash must change after removal");
    }

    // --- contains / get / len ---

    #[test]
    fn test_contains_and_get() {
        let mut map = ShaMap::new();
        let prefix = [0x01, 0x03, 0x03, 0x07];

        let mut idx = [0xAA; 32];
        idx[0] = 0xAA;
        map.add_item(idx, prefix, vec![42]);

        assert!(map.contains(&idx));
        let leaf = map.get(&idx).unwrap();
        assert_eq!(leaf.data, vec![42]);
        assert_eq!(leaf.index, idx);

        let missing = [0xBB; 32];
        assert!(!map.contains(&missing));
        assert!(map.get(&missing).is_none());
    }

    #[test]
    fn test_len_tracking() {
        let mut map = ShaMap::new();
        let prefix = [0x01, 0x03, 0x03, 0x07];
        assert_eq!(map.len(), 0);

        for i in 0u8..10 {
            let mut idx = [0u8; 32];
            idx[0] = i * 0x18;
            map.add_item(idx, prefix, vec![i]);
            assert_eq!(map.len(), (i + 1) as usize);
        }
    }

    // --- remove_item ---

    #[test]
    fn test_remove_single_item() {
        let mut map = ShaMap::new();
        let prefix = [0x01, 0x03, 0x03, 0x07];

        let idx = [0xAA; 32];
        map.add_item(idx, prefix, vec![1]);
        assert_eq!(map.len(), 1);

        assert!(map.remove_item(&idx));
        assert_eq!(map.len(), 0);
        assert!(map.is_empty());
        assert_eq!(map.hash(), ZERO_256);
    }

    #[test]
    fn test_remove_nonexistent_returns_false() {
        let mut map = ShaMap::new();
        let prefix = [0x01, 0x03, 0x03, 0x07];

        let idx = [0xAA; 32];
        map.add_item(idx, prefix, vec![1]);

        let missing = [0xBB; 32];
        assert!(!map.remove_item(&missing));
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn test_remove_collapses_inner_to_leaf() {
        let mut map = ShaMap::new();
        let prefix = [0x01, 0x03, 0x03, 0x07];

        // Two items sharing first nibble -> creates inner node at depth 1
        let mut idx1 = [0u8; 32];
        idx1[0] = 0xA0;
        let mut idx2 = [0u8; 32];
        idx2[0] = 0xA1;

        map.add_item(idx1, prefix, vec![1]);
        map.add_item(idx2, prefix, vec![2]);

        // Verify inner node exists
        assert!(map.root.get_child(0xA).unwrap().is_inner());

        // Remove one -> inner should collapse back to a leaf
        map.remove_item(&idx2);
        assert!(map.root.get_child(0xA).unwrap().is_leaf());
        assert_eq!(map.len(), 1);
        assert!(map.contains(&idx1));
    }

    #[test]
    fn test_add_remove_roundtrip_hash() {
        let prefix = [0x01, 0x03, 0x03, 0x07];

        let mut idx1 = [0u8; 32];
        idx1[0] = 0xAA;
        let mut idx2 = [0u8; 32];
        idx2[0] = 0xBB;

        // Build map with only idx1
        let mut map_one = ShaMap::new();
        map_one.add_item(idx1, prefix, vec![1]);
        let hash_one = map_one.hash();

        // Build map with both, then remove idx2
        let mut map_roundtrip = ShaMap::new();
        map_roundtrip.add_item(idx1, prefix, vec![1]);
        map_roundtrip.add_item(idx2, prefix, vec![2]);
        map_roundtrip.remove_item(&idx2);
        let hash_roundtrip = map_roundtrip.hash();

        assert_eq!(
            hash_one, hash_roundtrip,
            "add+remove roundtrip must restore original hash"
        );
    }

    // --- Proofs ---

    #[test]
    fn test_proof_existing_item() {
        let prefix = [0x01, 0x03, 0x03, 0x07];
        let mut map = ShaMap::new();

        let items: Vec<[u8; 32]> = (0..5)
            .map(|i| {
                let mut idx = [0u8; 32];
                idx[0] = i * 0x30;
                idx
            })
            .collect();

        for item in &items {
            map.add_item(*item, prefix, item.to_vec());
        }

        let root_hash = map.hash();

        for item in &items {
            let proof = map
                .extract_proof(item)
                .expect("proof should exist for inserted item");
            assert_eq!(proof.index, *item);
            assert!(
                verify_proof(&proof, &root_hash),
                "proof should verify against root hash"
            );
        }
    }

    #[test]
    fn test_proof_missing_item() {
        let prefix = [0x01, 0x03, 0x03, 0x07];
        let mut map = ShaMap::new();

        let mut idx = [0u8; 32];
        idx[0] = 0xAA;
        map.add_item(idx, prefix, vec![1, 2, 3]);

        let missing = [0xBB; 32];
        assert!(map.extract_proof(&missing).is_none());
    }

    #[test]
    fn test_proof_tampered_fails() {
        let prefix = [0x01, 0x03, 0x03, 0x07];
        let mut map = ShaMap::new();

        let items: Vec<[u8; 32]> = (0..3)
            .map(|i| {
                let mut idx = [0u8; 32];
                idx[0] = i * 0x50;
                idx
            })
            .collect();

        for item in &items {
            map.add_item(*item, prefix, item.to_vec());
        }

        let root_hash = map.hash();
        let mut proof = map.extract_proof(&items[0]).unwrap();
        proof.leaf_hash[0] ^= 0xFF;

        assert!(!verify_proof(&proof, &root_hash));
    }

    #[test]
    fn test_proof_wrong_root_fails() {
        let prefix = [0x01, 0x03, 0x03, 0x07];
        let mut map = ShaMap::new();

        let mut idx = [0u8; 32];
        idx[0] = 0xAA;
        map.add_item(idx, prefix, vec![1, 2, 3]);

        let proof = map.extract_proof(&idx).unwrap();
        let wrong_root = [0xFF; 32];
        assert!(!verify_proof(&proof, &wrong_root));
    }

    /// Proof-binding: a proof whose `index` does not match the nibble path
    /// recorded in `path` must be rejected. This is critical for
    /// `TRANSACTION_NO_METADATA` leaves where the leaf hash excludes the
    /// index: without binding, an attacker could pair any leaf_hash in the
    /// tree with an arbitrary index and have it verify against the root.
    #[test]
    fn test_proof_wrong_index_fails() {
        let prefix = hash_prefix::TRANSACTION_ID;
        let mut map = ShaMap::new();

        // Build a small tree of no-index leaves at distinct top-nibble slots
        // so each item lives on its own root branch.
        let items: Vec<[u8; 32]> = (0u8..4)
            .map(|i| {
                let mut idx = [0u8; 32];
                idx[0] = i.wrapping_mul(0x40); // 0x00, 0x40, 0x80, 0xC0
                idx
            })
            .collect();

        for item in &items {
            map.add_item_no_index(*item, prefix, item.to_vec());
        }

        let root_hash = map.hash();

        // Legit proof for items[0].
        let valid = map.extract_proof(&items[0]).unwrap();
        assert!(verify_proof(&valid, &root_hash));

        // Forge a proof: keep the valid path (which routes to items[0]) but
        // claim it proves `items[1]`. Because `leaf_hash` for no-index leaves
        // omits the index, the forged proof would re-hash to the same root
        // without the index-binding check, so verification must reject it.
        let forged = ShaMapProof {
            index: items[1],
            leaf_hash: valid.leaf_hash,
            path: valid
                .path
                .iter()
                .map(|lvl| ProofLevel {
                    nibble: lvl.nibble,
                    sibling_hashes: lvl.sibling_hashes,
                })
                .collect(),
        };

        assert!(
            !verify_proof(&forged, &root_hash),
            "proof with wrong index must be rejected"
        );
    }

    // --- Fuzz-style tests ---

    #[test]
    fn test_fuzz_order_independence() {
        let prefix = [0x01, 0x03, 0x03, 0x07];

        let items: Vec<[u8; 32]> = (0u8..20)
            .map(|i| {
                let mut idx = [0u8; 32];
                idx[0] = i.wrapping_mul(13);
                idx[1] = i.wrapping_mul(37);
                idx[2] = i.wrapping_mul(71);
                idx
            })
            .collect();

        let mut map1 = ShaMap::new();
        for item in &items {
            map1.add_item(*item, prefix, item.to_vec());
        }
        let hash1 = map1.hash();

        let mut map2 = ShaMap::new();
        for item in items.iter().rev() {
            map2.add_item(*item, prefix, item.to_vec());
        }
        let hash2 = map2.hash();

        let mut map3 = ShaMap::new();
        for item in items.iter().step_by(2) {
            map3.add_item(*item, prefix, item.to_vec());
        }
        for item in items.iter().skip(1).step_by(2) {
            map3.add_item(*item, prefix, item.to_vec());
        }
        let hash3 = map3.hash();

        assert_eq!(hash1, hash2, "reverse order must produce same hash");
        assert_eq!(hash1, hash3, "interleaved order must produce same hash");
    }

    #[test]
    fn test_fuzz_proof_verification() {
        let prefix = [0x01, 0x03, 0x03, 0x07];

        let items: Vec<[u8; 32]> = (0u8..15)
            .map(|i| {
                let mut idx = [0u8; 32];
                idx[0] = i.wrapping_mul(17);
                idx[1] = i.wrapping_mul(43);
                idx
            })
            .collect();

        let mut map = ShaMap::new();
        for item in &items {
            map.add_item(*item, prefix, item.to_vec());
        }
        let root_hash = map.hash();

        for item in &items {
            let proof = map.extract_proof(item).expect("proof must exist");
            assert!(verify_proof(&proof, &root_hash));
        }

        for i in 20u8..30 {
            let mut idx = [0u8; 32];
            idx[0] = i.wrapping_mul(17);
            idx[1] = i.wrapping_mul(43);
            if !items.contains(&idx) {
                assert!(map.extract_proof(&idx).is_none());
            }
        }
    }

    #[test]
    fn test_fuzz_tampered_proof_fails() {
        let prefix = [0x01, 0x03, 0x03, 0x07];

        let items: Vec<[u8; 32]> = (0u8..10)
            .map(|i| {
                let mut idx = [0u8; 32];
                idx[0] = i.wrapping_mul(23);
                idx[1] = i.wrapping_mul(59);
                idx
            })
            .collect();

        let mut map = ShaMap::new();
        for item in &items {
            map.add_item(*item, prefix, item.to_vec());
        }
        let root_hash = map.hash();

        for (i, item) in items.iter().enumerate() {
            let mut proof = map.extract_proof(item).unwrap();

            match i % 3 {
                0 => {
                    proof.leaf_hash[0] ^= 0xFF;
                }
                1 => {
                    if !proof.path.is_empty() {
                        proof.path[0].sibling_hashes[0][0] ^= 0xFF;
                    } else {
                        proof.leaf_hash[0] ^= 0xFF;
                    }
                }
                _ => {
                    if !proof.path.is_empty() {
                        proof.path[0].nibble = (proof.path[0].nibble + 1) % 16;
                    } else {
                        proof.leaf_hash[0] ^= 0xFF;
                    }
                }
            }

            assert!(!verify_proof(&proof, &root_hash));
        }
    }

    #[test]
    fn test_fuzz_remove_and_verify() {
        let prefix = [0x01, 0x03, 0x03, 0x07];

        let items: Vec<[u8; 32]> = (0u8..20)
            .map(|i| {
                let mut idx = [0u8; 32];
                idx[0] = i.wrapping_mul(11);
                idx[1] = i.wrapping_mul(31);
                idx
            })
            .collect();

        let mut map = ShaMap::new();
        for item in &items {
            map.add_item(*item, prefix, item.to_vec());
        }

        // Remove every other item and verify the remaining ones
        for item in items.iter().step_by(2) {
            assert!(map.remove_item(item));
        }

        assert_eq!(map.len(), 10);

        let root_hash = map.hash();

        // Remaining items should have valid proofs
        for item in items.iter().skip(1).step_by(2) {
            assert!(map.contains(item));
            let proof = map.extract_proof(item).expect("proof must exist");
            assert!(verify_proof(&proof, &root_hash));
        }

        // Removed items should not be found
        for item in items.iter().step_by(2) {
            assert!(!map.contains(item));
            assert!(map.extract_proof(item).is_none());
        }
    }

    #[test]
    fn test_hamt_sparse_inner_efficiency() {
        // An inner node with 2 children should have children.len() == 2
        let mut map = ShaMap::new();
        let prefix = [0x01, 0x03, 0x03, 0x07];

        let mut idx1 = [0u8; 32];
        idx1[0] = 0x00;
        let mut idx2 = [0u8; 32];
        idx2[0] = 0xF0;

        map.add_item(idx1, prefix, vec![1]);
        map.add_item(idx2, prefix, vec![2]);

        // Root should have exactly 2 children in its compact Vec
        assert_eq!(map.root.branch_count(), 2);
        assert_eq!(map.root.children.len(), 2);
    }

    #[test]
    fn test_leaf_hash_caching() {
        let leaf = ShaMapLeaf::new([0xAB; 32], [0x53, 0x4E, 0x44, 0x00], vec![1, 2, 3]);

        // First call computes
        let h1 = leaf.hash();
        // Second call returns cached
        let h2 = leaf.hash();
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_inner_hash_matches_streaming() {
        // Verify single-buffer hash matches the streaming approach
        let mut map = ShaMap::new();
        let prefix = [0x01, 0x03, 0x03, 0x07];

        for i in 0u8..5 {
            let mut idx = [0u8; 32];
            idx[0] = i * 0x30;
            map.add_item(idx, prefix, vec![i]);
        }

        let single_buf_hash = map.hash();

        // Compute with streaming approach
        let mut h = Sha512Half::new();
        h.update(&hash_prefix::INNER_NODE);
        for n in 0..16u8 {
            let child_hash = match map.root.get_child(n as usize) {
                Some(node) => node.hash(),
                None => ZERO_256,
            };
            h.update(&child_hash);
        }
        let streaming_hash = h.finish();

        assert_eq!(single_buf_hash, streaming_hash);
    }

    #[test]
    fn test_remove_from_deep_tree() {
        let prefix = [0x01, 0x03, 0x03, 0x07];
        let mut map = ShaMap::new();

        // Items sharing many nibbles to create deep tree
        let mut idx1 = [0u8; 32];
        idx1[0] = 0xAB;
        idx1[1] = 0xCD;
        idx1[2] = 0xE0;

        let mut idx2 = [0u8; 32];
        idx2[0] = 0xAB;
        idx2[1] = 0xCD;
        idx2[2] = 0xE1;

        let mut idx3 = [0u8; 32];
        idx3[0] = 0xAB;
        idx3[1] = 0xCD;
        idx3[2] = 0xF0;

        map.add_item(idx1, prefix, vec![1]);
        map.add_item(idx2, prefix, vec![2]);
        map.add_item(idx3, prefix, vec![3]);
        assert_eq!(map.len(), 3);

        // Remove one of the deep items
        assert!(map.remove_item(&idx1));
        assert_eq!(map.len(), 2);
        assert!(!map.contains(&idx1));
        assert!(map.contains(&idx2));
        assert!(map.contains(&idx3));

        // Hash should still be valid
        let root_hash = map.hash();
        assert_ne!(root_hash, ZERO_256);

        // Proofs should still verify
        for idx in [idx2, idx3] {
            let proof = map.extract_proof(&idx).unwrap();
            assert!(verify_proof(&proof, &root_hash));
        }
    }

    // -----------------------------------------------------------------------
    // xrpl.js compatibility tests — ported from:
    //   packages/xrpl/test/shamap.test.ts
    //   packages/ripple-binary-codec/test/shamap.test.ts
    // -----------------------------------------------------------------------

    /// Helper: decode a hex string to a 32-byte array.
    fn hex_to_32(hex: &str) -> [u8; 32] {
        let bytes = hex::decode(hex).unwrap();
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        arr
    }

    /// xrpl.js ShaMap test: 8 items inserted incrementally with
    /// TRANSACTION_NO_METADATA node type (hash = sha512half(TRANSACTION_ID || data),
    /// no index in hash). Expected root hashes verified after each insertion.
    ///
    /// Source: packages/xrpl/test/shamap.test.ts
    #[test]
    fn test_xrpljs_shamap_incremental_hashes() {
        let keys = [
            "b92891fe4ef6cee585fdc6fda1e09eb4d386363158ec3321b8123e5a772c6ca8",
            "b92881fe4ef6cee585fdc6fda1e09eb4d386363158ec3321b8123e5a772c6ca8",
            "b92691fe4ef6cee585fdc6fda1e09eb4d386363158ec3321b8123e5a772c6ca8",
            "b92791fe4ef6cee585fdc6fda1e09eb4d386363158ec3321b8123e5a772c6ca8",
            "b91891fe4ef6cee585fdc6fda1e09eb4d386363158ec3321b8123e5a772c6ca8",
            "b99891fe4ef6cee585fdc6fda1e09eb4d386363158ec3321b8123e5a772c6ca8",
            "f22891fe4ef6cee585fdc6fda1e09eb4d386363158ec3321b8123e5a772c6ca8",
            "292891fe4ef6cee585fdc6fda1e09eb4d386363158ec3321b8123e5a772c6ca8",
        ];

        let expected_hashes = [
            "B7387CFEA0465759ADC718E8C42B52D2309D179B326E239EB5075C64B6281F7F",
            "FBC195A9592A54AB44010274163CB6BA95F497EC5BA0A8831845467FB2ECE266",
            "4E7D2684B65DFD48937FFB775E20175C43AF0C94066F7D5679F51AE756795B75",
            "7A2F312EB203695FFD164E038E281839EEF06A1B99BFC263F3CECC6C74F93E07",
            "395A6691A372387A703FB0F2C6D2C405DAF307D0817F8F0E207596462B0E3A3E",
            "D044C0A696DE3169CC70AE216A1564D69DE96582865796142CE7D98A84D9DDE4",
            "76DCC77C4027309B5A91AD164083264D70B77B5E43E08AEDA5EBF94361143615",
            "DF4220E93ADC6F5569063A01B4DC79F8DB9553B6A3222ADE23DEA02BBE7230E5",
        ];

        // Data for item i: byte i repeated 32 times (matching intToVuc(i))
        let tx_id_prefix = hash_prefix::TRANSACTION_ID;

        let mut map = ShaMap::new();

        for (i, key_hex) in keys.iter().enumerate() {
            let key = hex_to_32(key_hex);
            let data: Vec<u8> = vec![i as u8; 32];

            // TRANSACTION_NO_METADATA: hash = sha512half(TRANSACTION_ID || data)
            map.add_item_no_index(key, tx_id_prefix, data);

            let root_hash = map.hash();
            let expected = hex_to_32(expected_hashes[i]);

            assert_eq!(
                root_hash, expected,
                "root hash mismatch after inserting item {} (key {})",
                i, key_hex
            );
        }
    }

    /// xrpl.js test: empty ShaMap hashes to ZERO_256.
    /// Source: packages/xrpl/test/shamap.test.ts
    #[test]
    fn test_xrpljs_empty_shamap_hash() {
        let map = ShaMap::new();
        assert_eq!(map.hash(), ZERO_256);
    }

    /// Test the no_index leaf variant produces different hash than with-index.
    #[test]
    fn test_no_index_leaf_differs_from_with_index() {
        let index = [0xAA; 32];
        let prefix = hash_prefix::TRANSACTION_ID;
        let data = vec![1, 2, 3];

        let leaf_with = ShaMapLeaf::new(index, prefix, data.clone());
        let leaf_without = ShaMapLeaf::new_no_index(index, prefix, data);

        assert_ne!(
            leaf_with.hash(),
            leaf_without.hash(),
            "with-index and no-index leaf hashes must differ"
        );
    }

    // -----------------------------------------------------------------------
    // Edge case coverage tests
    // -----------------------------------------------------------------------

    /// remove_item returns false when the leaf at the branch has a different index
    /// (collision path — two items share nibbles but different full index).
    #[test]
    fn test_remove_wrong_leaf_at_branch() {
        let prefix = [0x01, 0x03, 0x03, 0x07];
        let mut map = ShaMap::new();

        let mut idx1 = [0u8; 32];
        idx1[0] = 0xAA;

        map.add_item(idx1, prefix, vec![1]);

        // Try to remove an item that shares the same nibble path but has a
        // different full index (differs deep in the key, past tree depth).
        let mut wrong = [0u8; 32];
        wrong[0] = 0xAA;
        wrong[31] = 0xFF; // different at the last byte

        assert!(
            !map.remove_item(&wrong),
            "must return false for non-matching full index"
        );
        assert_eq!(map.len(), 1);
    }

    /// remove_item returns false when recursing into an inner node that
    /// doesn't contain the target (item not present in subtree).
    #[test]
    fn test_remove_missing_from_inner() {
        let prefix = [0x01, 0x03, 0x03, 0x07];
        let mut map = ShaMap::new();

        // Create an inner node at branch 0xA with two leaves
        let mut idx1 = [0u8; 32];
        idx1[0] = 0xA0;
        let mut idx2 = [0u8; 32];
        idx2[0] = 0xA1;

        map.add_item(idx1, prefix, vec![1]);
        map.add_item(idx2, prefix, vec![2]);

        // Try to remove an item that would route to the same inner node
        // but isn't present (different second nibble).
        let mut missing = [0u8; 32];
        missing[0] = 0xA2;

        assert!(
            !map.remove_item(&missing),
            "must return false for item not in subtree"
        );
        assert_eq!(map.len(), 2);
    }

    /// get() returns None when the leaf at the target branch has a different
    /// full index (hash collision at the tree level).
    #[test]
    fn test_get_wrong_leaf_at_branch() {
        let prefix = [0x01, 0x03, 0x03, 0x07];
        let mut map = ShaMap::new();

        let mut idx = [0u8; 32];
        idx[0] = 0xBB;
        map.add_item(idx, prefix, vec![42]);

        // Query for an index that shares the full nibble path but differs
        // deep in the key (past tree depth for a single-item tree).
        let mut query = [0u8; 32];
        query[0] = 0xBB;
        query[31] = 0x01; // different last byte

        assert!(
            map.get(&query).is_none(),
            "get must return None for non-matching full index"
        );
    }

    /// get() recurses into an inner node to find a nested leaf.
    #[test]
    fn test_get_through_inner_node() {
        let prefix = [0x01, 0x03, 0x03, 0x07];
        let mut map = ShaMap::new();

        // Force inner node creation at branch 0xC
        let mut idx1 = [0u8; 32];
        idx1[0] = 0xC0;
        let mut idx2 = [0u8; 32];
        idx2[0] = 0xC1;

        map.add_item(idx1, prefix, vec![10]);
        map.add_item(idx2, prefix, vec![20]);

        // get() must traverse the inner node at branch 0xC
        let leaf = map.get(&idx2).expect("must find leaf through inner node");
        assert_eq!(leaf.data, vec![20]);
    }

    /// extract_proof returns None when the target is in an inner subtree
    /// but the leaf doesn't match (path exists but item absent).
    #[test]
    fn test_proof_missing_from_inner_subtree() {
        let prefix = [0x01, 0x03, 0x03, 0x07];
        let mut map = ShaMap::new();

        // Create inner node at branch 0xD
        let mut idx1 = [0u8; 32];
        idx1[0] = 0xD0;
        let mut idx2 = [0u8; 32];
        idx2[0] = 0xD1;

        map.add_item(idx1, prefix, vec![1]);
        map.add_item(idx2, prefix, vec![2]);

        // Query for an item that routes into the same inner but isn't there
        let mut missing = [0u8; 32];
        missing[0] = 0xD2;

        assert!(
            map.extract_proof(&missing).is_none(),
            "proof must be None for item not in inner subtree"
        );
    }
}
