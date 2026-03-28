//! Fuzz target: proof integrity.
//!
//! For any set of items:
//! 1. Every inserted item has a valid Merkle inclusion proof.
//! 2. Flipping any bit in the leaf hash invalidates the proof.
//! 3. Items not in the tree have no proof.

#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use xrpl::core::shamap::{verify_proof, ShaMap};

#[derive(Arbitrary, Debug)]
struct FuzzInput {
    items: Vec<[u8; 32]>,
    missing: Vec<[u8; 32]>,
}

fuzz_target!(|input: FuzzInput| {
    if input.items.is_empty() || input.items.len() > 200 {
        return;
    }

    let prefix = [0x53, 0x4E, 0x44, 0x00]; // TRANSACTION

    // Deduplicate
    let mut seen = std::collections::HashSet::new();
    let items: Vec<[u8; 32]> = input
        .items
        .iter()
        .copied()
        .filter(|idx| seen.insert(*idx))
        .collect();

    if items.is_empty() {
        return;
    }

    let mut map = ShaMap::new();
    for idx in &items {
        map.add_item(*idx, prefix, idx.to_vec());
    }
    let root_hash = map.hash();

    // Property 1: every item has a valid proof
    for idx in &items {
        let proof = map
            .extract_proof(idx)
            .expect("proof must exist for inserted item");
        assert!(
            verify_proof(&proof, &root_hash),
            "valid proof must verify"
        );

        // Property 2: flipping one bit in the leaf hash must invalidate
        let mut tampered = proof;
        tampered.leaf_hash[0] ^= 1;
        assert!(
            !verify_proof(&tampered, &root_hash),
            "tampered proof must not verify"
        );
    }

    // Property 3: items not in the tree have no proof
    for missing_idx in &input.missing {
        if !seen.contains(missing_idx) {
            assert!(
                map.extract_proof(missing_idx).is_none(),
                "missing item must have no proof"
            );
        }
    }
});
