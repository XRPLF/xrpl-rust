//! ShaMap: a hash tree (radix-16 Merkle trie) used in the XRP Ledger.
//!
//! This implements the full tree structure matching the xrpl.js ShaMap class
//! hierarchy. Each leaf is keyed by a 256-bit index. Inner nodes have 16
//! branches (one per hex nibble). The tree is hashed bottom-up to produce
//! a single root hash that commits to all entries.

use alloc::boxed::Box;
use alloc::vec::Vec;

use super::hash_prefix;
use super::sha512half::Sha512Half;

/// A 256-bit ShaMap key (index).
pub type ShaMapIndex = [u8; 32];

/// A 256-bit hash value.
pub type Hash256 = [u8; 32];

/// The zero hash, used for empty branches.
pub const ZERO_256: Hash256 = [0u8; 32];

/// Extract the nibble (0..15) at the given depth from an index.
///
/// Depth 0 is the high nibble of byte 0, depth 1 is the low nibble of byte 0,
/// depth 2 is the high nibble of byte 1, etc.
fn nibble(index: &ShaMapIndex, depth: usize) -> usize {
    let byte = index[depth / 2];
    if depth.is_multiple_of(2) {
        (byte >> 4) as usize
    } else {
        (byte & 0x0F) as usize
    }
}

/// A leaf node in the ShaMap, containing the item data.
pub struct ShaMapLeaf {
    /// The 256-bit index (key) for this leaf.
    pub index: ShaMapIndex,
    /// The hash prefix to use when hashing this leaf.
    pub hash_prefix: [u8; 4],
    /// The serialized item data.
    pub data: Vec<u8>,
}

impl ShaMapLeaf {
    /// Compute the hash of this leaf: `sha512half(prefix || data || index)`.
    pub fn hash(&self) -> Hash256 {
        let mut h = Sha512Half::new();
        h.update(&self.hash_prefix);
        h.update(&self.data);
        h.update(&self.index);
        h.finish()
    }
}

/// An inner node in the ShaMap with 16 branches.
pub struct ShaMapInner {
    depth: usize,
    branches: [Option<Box<ShaMapNode>>; 16],
}

/// A node in the ShaMap tree -- either a leaf or an inner node.
pub enum ShaMapNode {
    /// A leaf containing item data.
    Leaf(ShaMapLeaf),
    /// An inner node with up to 16 children.
    Inner(ShaMapInner),
}

impl ShaMapInner {
    /// Create a new empty inner node at the given depth.
    pub fn new(depth: usize) -> Self {
        ShaMapInner {
            depth,
            branches: Default::default(),
        }
    }

    /// Returns true if all 16 branches are empty.
    pub fn is_empty(&self) -> bool {
        self.branches.iter().all(|b| b.is_none())
    }

    /// Compute the hash of this inner node.
    ///
    /// An empty inner node hashes to `ZERO_256`. Otherwise the hash is
    /// `sha512half(INNER_NODE_PREFIX || h0 || h1 || ... || h15)` where
    /// each `hi` is the child hash or `ZERO_256` if the branch is empty.
    pub fn hash(&self) -> Hash256 {
        if self.is_empty() {
            return ZERO_256;
        }

        let mut h = Sha512Half::new();
        h.update(&hash_prefix::INNER_NODE);

        for branch in &self.branches {
            let child_hash = match branch {
                Some(node) => node.hash(),
                None => ZERO_256,
            };
            h.update(&child_hash);
        }

        h.finish()
    }

    /// Insert a leaf into this inner node, creating sub-trees as needed.
    ///
    /// This follows the xrpl.js insertion algorithm:
    /// - If the target branch is empty, place the leaf there.
    /// - If the target branch holds a leaf, create a new inner node at depth+1,
    ///   re-insert the existing leaf, then insert the new leaf.
    /// - If the target branch holds an inner node, recurse into it.
    pub fn add_item(&mut self, leaf: ShaMapLeaf) {
        let slot = nibble(&leaf.index, self.depth);
        let branch = self.branches[slot].take();

        match branch {
            None => {
                self.branches[slot] = Some(Box::new(ShaMapNode::Leaf(leaf)));
            }
            Some(existing) => match *existing {
                ShaMapNode::Leaf(existing_leaf) => {
                    // Collision at this depth: create a deeper inner node
                    let mut new_inner = ShaMapInner::new(self.depth + 1);
                    new_inner.add_item(existing_leaf);
                    new_inner.add_item(leaf);
                    self.branches[slot] = Some(Box::new(ShaMapNode::Inner(new_inner)));
                }
                ShaMapNode::Inner(mut inner) => {
                    inner.add_item(leaf);
                    self.branches[slot] = Some(Box::new(ShaMapNode::Inner(inner)));
                }
            },
        }
    }
}

impl ShaMapNode {
    /// Compute the hash of this node (delegates to leaf or inner).
    fn hash(&self) -> Hash256 {
        match self {
            ShaMapNode::Leaf(leaf) => leaf.hash(),
            ShaMapNode::Inner(inner) => inner.hash(),
        }
    }
}

/// A level in a ShaMap inclusion proof.
///
/// Contains the nibble taken at this depth and the hashes of all 16 siblings
/// at that inner node level.
pub struct ProofLevel {
    /// Which branch (0..15) was taken at this level.
    pub nibble: u8,
    /// Hashes of all 16 branches at this inner node. The branch at `nibble`
    /// is NOT used during verification (it is replaced by the hash propagated
    /// from the level below).
    pub sibling_hashes: [Hash256; 16],
}

/// An inclusion proof for a single leaf in the ShaMap.
pub struct ShaMapProof {
    /// The index of the leaf being proved.
    pub index: ShaMapIndex,
    /// The hash of the leaf.
    pub leaf_hash: Hash256,
    /// The proof path from the leaf up to the root.
    pub path: Vec<ProofLevel>,
}

/// Verify a ShaMap inclusion proof against an expected root hash.
///
/// Recomputes the root hash from the leaf hash and the sibling hashes at each
/// level, then checks if it matches `expected_root`.
pub fn verify_proof(proof: &ShaMapProof, expected_root: &Hash256) -> bool {
    let mut current_hash = proof.leaf_hash;

    // Walk the path from leaf toward root (path is stored leaf-to-root)
    for level in proof.path.iter().rev() {
        let mut h = Sha512Half::new();
        h.update(&hash_prefix::INNER_NODE);

        for i in 0..16 {
            if i == level.nibble as usize {
                h.update(&current_hash);
            } else {
                h.update(&level.sibling_hashes[i]);
            }
        }

        current_hash = h.finish();
    }

    current_hash == *expected_root
}

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

    /// Add an item to the ShaMap.
    pub fn add_item(&mut self, index: ShaMapIndex, hash_prefix: [u8; 4], data: Vec<u8>) {
        let leaf = ShaMapLeaf {
            index,
            hash_prefix,
            data,
        };
        self.root.add_item(leaf);
    }

    /// Compute the root hash of the ShaMap.
    pub fn hash(&self) -> Hash256 {
        self.root.hash()
    }

    /// Extract an inclusion proof for the item at `target`.
    ///
    /// Returns `None` if the target index is not in the tree.
    pub fn extract_proof(&self, target: &ShaMapIndex) -> Option<ShaMapProof> {
        let mut path = Vec::new();
        let leaf_hash = Self::collect_proof(&self.root, target, &mut path)?;

        Some(ShaMapProof {
            index: *target,
            leaf_hash,
            path,
        })
    }

    /// Recursively walk the tree to find `target`, collecting proof levels.
    ///
    /// Returns the leaf hash if found. Proof levels are pushed in root-to-leaf
    /// order (which is how `verify_proof` expects them, reversed).
    fn collect_proof(
        inner: &ShaMapInner,
        target: &ShaMapIndex,
        path: &mut Vec<ProofLevel>,
    ) -> Option<Hash256> {
        let slot = nibble(target, inner.depth);

        // Collect sibling hashes at this level
        let mut sibling_hashes = [ZERO_256; 16];
        for (i, branch) in inner.branches.iter().enumerate() {
            sibling_hashes[i] = match branch {
                Some(node) => node.hash(),
                None => ZERO_256,
            };
        }

        path.push(ProofLevel {
            nibble: slot as u8,
            sibling_hashes,
        });

        match &inner.branches[slot] {
            None => {
                path.pop();
                None
            }
            Some(node) => match node.as_ref() {
                ShaMapNode::Leaf(leaf) => {
                    if leaf.index == *target {
                        Some(leaf.hash())
                    } else {
                        path.pop();
                        None
                    }
                }
                ShaMapNode::Inner(child_inner) => Self::collect_proof(child_inner, target, path),
            },
        }
    }
}

impl Default for ShaMap {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn test_nibble_extraction() {
        // Index starting with 0xAB...
        let mut idx = [0u8; 32];
        idx[0] = 0xAB;
        idx[1] = 0xCD;

        assert_eq!(nibble(&idx, 0), 0xA); // high nibble of byte 0
        assert_eq!(nibble(&idx, 1), 0xB); // low nibble of byte 0
        assert_eq!(nibble(&idx, 2), 0xC); // high nibble of byte 1
        assert_eq!(nibble(&idx, 3), 0xD); // low nibble of byte 1
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

    #[test]
    fn test_empty_shamap_is_zero() {
        let map = ShaMap::new();
        assert_eq!(map.hash(), ZERO_256);
    }

    #[test]
    fn test_single_item_hash() {
        let mut map = ShaMap::new();
        let index = [0u8; 32];
        let data = vec![1, 2, 3, 4];
        let prefix = [0x01, 0x03, 0x03, 0x07];

        map.add_item(index, prefix, data.clone());

        // Manually compute expected leaf hash
        let leaf = ShaMapLeaf {
            index,
            hash_prefix: prefix,
            data: data.clone(),
        };
        let leaf_hash = leaf.hash();

        // Single-item tree: root inner hashes over INNER_NODE prefix + 16 child hashes
        // Only branch 0 (nibble 0 at depth 0) is occupied
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

    #[test]
    fn test_two_items_different_nibbles() {
        let mut map = ShaMap::new();

        // First item: index starts with 0x00
        let mut idx1 = [0u8; 32];
        idx1[0] = 0x00;

        // Second item: index starts with 0x10 (different first nibble)
        let mut idx2 = [0u8; 32];
        idx2[0] = 0x10;

        let prefix = [0x01, 0x03, 0x03, 0x07];
        map.add_item(idx1, prefix, vec![1]);
        map.add_item(idx2, prefix, vec![2]);

        let hash = map.hash();
        assert_ne!(hash, ZERO_256, "non-empty tree must not hash to zero");
    }

    #[test]
    fn test_two_items_same_first_nibble_creates_inner() {
        let mut map = ShaMap::new();

        // Both start with 0xA but differ at second nibble
        let mut idx1 = [0u8; 32];
        idx1[0] = 0xA0;
        let mut idx2 = [0u8; 32];
        idx2[0] = 0xA1;

        let prefix = [0x01, 0x03, 0x03, 0x07];
        map.add_item(idx1, prefix, vec![1]);
        map.add_item(idx2, prefix, vec![2]);

        let hash = map.hash();
        assert_ne!(hash, ZERO_256);

        // Verify the structure: root branch 0xA should be an inner node
        match &map.root.branches[0xA] {
            Some(node) => match node.as_ref() {
                ShaMapNode::Inner(inner) => {
                    assert_eq!(inner.depth, 1);
                    // Branch 0 and 1 at depth 1 should be leaves
                    assert!(inner.branches[0].is_some());
                    assert!(inner.branches[1].is_some());
                }
                ShaMapNode::Leaf(_) => panic!("expected inner node, got leaf"),
            },
            None => panic!("expected branch 0xA to be occupied"),
        }
    }

    #[test]
    fn test_order_independence() {
        let prefix = [0x01, 0x03, 0x03, 0x07];

        // The items from the xrpl.js test, padded to 64 hex chars = 32 bytes
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

        // Insert in original order
        let mut map1 = ShaMap::new();
        for item in &items {
            map1.add_item(*item, prefix, item.to_vec());
        }
        let hash1 = map1.hash();

        // Insert in reverse order
        let mut map2 = ShaMap::new();
        for item in items.iter().rev() {
            map2.add_item(*item, prefix, item.to_vec());
        }
        let hash2 = map2.hash();

        assert_eq!(hash1, hash2, "hash must be order-independent");
        assert_ne!(hash1, ZERO_256, "non-empty tree must not hash to zero");
    }

    #[test]
    fn test_leaf_hash_correctness() {
        let index = [0xAB; 32];
        let prefix = [0x53, 0x4E, 0x44, 0x00]; // TRANSACTION prefix
        let data = vec![0x01, 0x02, 0x03];

        let leaf = ShaMapLeaf {
            index,
            hash_prefix: prefix,
            data: data.clone(),
        };

        // Manually compute: sha512half(prefix || data || index)
        let mut h = Sha512Half::new();
        h.update(&prefix);
        h.update(&data);
        h.update(&index);
        let expected = h.finish();

        assert_eq!(leaf.hash(), expected);
    }

    #[test]
    fn test_depth_verification() {
        // Force tree to go multiple levels deep with items sharing long prefixes
        let prefix = [0x01, 0x03, 0x03, 0x07];
        let mut map = ShaMap::new();

        // Items that share first 3 nibbles (0xABC) but differ at 4th
        let mut idx1 = [0u8; 32];
        idx1[0] = 0xAB;
        idx1[1] = 0xC0;

        let mut idx2 = [0u8; 32];
        idx2[0] = 0xAB;
        idx2[1] = 0xC1;

        map.add_item(idx1, prefix, vec![1]);
        map.add_item(idx2, prefix, vec![2]);

        // Navigate: root -> branch A -> branch B -> branch C -> should have two leaves
        let branch_a = match &map.root.branches[0xA] {
            Some(node) => match node.as_ref() {
                ShaMapNode::Inner(inner) => inner,
                _ => panic!("expected inner at depth 1"),
            },
            None => panic!("expected branch A"),
        };
        assert_eq!(branch_a.depth, 1);

        let branch_b = match &branch_a.branches[0xB] {
            Some(node) => match node.as_ref() {
                ShaMapNode::Inner(inner) => inner,
                _ => panic!("expected inner at depth 2"),
            },
            None => panic!("expected branch B"),
        };
        assert_eq!(branch_b.depth, 2);

        let branch_c = match &branch_b.branches[0xC] {
            Some(node) => match node.as_ref() {
                ShaMapNode::Inner(inner) => inner,
                _ => panic!("expected inner at depth 3"),
            },
            None => panic!("expected branch C"),
        };
        assert_eq!(branch_c.depth, 3);

        // At depth 3, branches 0 and 1 should be leaves
        assert!(matches!(
            branch_c.branches[0].as_ref().map(|n| n.as_ref()),
            Some(ShaMapNode::Leaf(_))
        ));
        assert!(matches!(
            branch_c.branches[1].as_ref().map(|n| n.as_ref()),
            Some(ShaMapNode::Leaf(_))
        ));
    }

    #[test]
    fn test_proof_existing_item() {
        let prefix = [0x01, 0x03, 0x03, 0x07];
        let mut map = ShaMap::new();

        let items: Vec<[u8; 32]> = (0..5)
            .map(|i| {
                let mut idx = [0u8; 32];
                idx[0] = i * 0x30; // spread across different first nibbles
                idx
            })
            .collect();

        for item in &items {
            map.add_item(*item, prefix, item.to_vec());
        }

        let root_hash = map.hash();

        // Extract and verify proof for each item
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

        // Try to get proof for an item not in the tree
        let missing = [0xBB; 32];
        assert!(
            map.extract_proof(&missing).is_none(),
            "should return None for missing item"
        );
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

        // Tamper with the leaf hash
        proof.leaf_hash[0] ^= 0xFF;

        assert!(
            !verify_proof(&proof, &root_hash),
            "tampered proof must not verify"
        );
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
        assert!(
            !verify_proof(&proof, &wrong_root),
            "proof must not verify against wrong root"
        );
    }

    #[test]
    fn test_fuzz_order_independence() {
        use alloc::vec::Vec;

        let prefix = [0x01, 0x03, 0x03, 0x07];

        // Generate 20 items with pseudo-random indices
        let items: Vec<[u8; 32]> = (0u8..20)
            .map(|i| {
                let mut idx = [0u8; 32];
                // Use a simple deterministic spread
                idx[0] = i.wrapping_mul(13);
                idx[1] = i.wrapping_mul(37);
                idx[2] = i.wrapping_mul(71);
                idx
            })
            .collect();

        // Forward order
        let mut map1 = ShaMap::new();
        for item in &items {
            map1.add_item(*item, prefix, item.to_vec());
        }
        let hash1 = map1.hash();

        // Reverse order
        let mut map2 = ShaMap::new();
        for item in items.iter().rev() {
            map2.add_item(*item, prefix, item.to_vec());
        }
        let hash2 = map2.hash();

        // Interleaved order (evens then odds)
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

        // Generate items
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

        // Every item should have a valid proof
        for item in &items {
            let proof = map
                .extract_proof(item)
                .expect("proof must exist for inserted item");
            assert!(
                verify_proof(&proof, &root_hash),
                "proof must verify for item {:?}",
                &item[..4]
            );
        }

        // Non-existent items should have no proof
        for i in 20u8..30 {
            let mut idx = [0u8; 32];
            idx[0] = i.wrapping_mul(17);
            idx[1] = i.wrapping_mul(43);
            // Only check if this index wasn't accidentally in our set
            if !items.contains(&idx) {
                assert!(
                    map.extract_proof(&idx).is_none(),
                    "non-existent item must have no proof"
                );
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

        // Tamper with each proof in different ways
        for (i, item) in items.iter().enumerate() {
            let mut proof = map.extract_proof(item).unwrap();

            match i % 3 {
                0 => {
                    // Tamper leaf hash
                    proof.leaf_hash[0] ^= 0xFF;
                }
                1 => {
                    // Tamper a sibling hash in the path
                    if !proof.path.is_empty() {
                        proof.path[0].sibling_hashes[0][0] ^= 0xFF;
                    } else {
                        proof.leaf_hash[0] ^= 0xFF;
                    }
                }
                _ => {
                    // Tamper the nibble
                    if !proof.path.is_empty() {
                        proof.path[0].nibble = (proof.path[0].nibble + 1) % 16;
                    } else {
                        proof.leaf_hash[0] ^= 0xFF;
                    }
                }
            }

            assert!(
                !verify_proof(&proof, &root_hash),
                "tampered proof must not verify"
            );
        }
    }

    #[test]
    fn test_empty_inner_hash_is_zero() {
        let inner = ShaMapInner::new(0);
        assert!(inner.is_empty());
        assert_eq!(inner.hash(), ZERO_256);
    }

    #[test]
    fn test_default_shamap() {
        let map = ShaMap::default();
        assert_eq!(map.hash(), ZERO_256);
    }
}
