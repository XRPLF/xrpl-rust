//! Fuzz target: hash caching consistency.
//!
//! Perform a random sequence of add/remove operations. After each mutation,
//! verify that the cached hash equals a freshly-built tree's hash (built
//! from the same logical contents).

#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use xrpl::core::shamap::ShaMap;

#[derive(Arbitrary, Debug)]
enum Op {
    Add([u8; 32]),
    Remove([u8; 32]),
}

#[derive(Arbitrary, Debug)]
struct FuzzInput {
    ops: Vec<Op>,
}

fuzz_target!(|input: FuzzInput| {
    if input.ops.is_empty() || input.ops.len() > 300 {
        return;
    }

    let prefix = [0x4D, 0x4C, 0x4E, 0x00]; // ACCOUNT_STATE_ENTRY

    let mut map = ShaMap::new();
    // Track the current set of items to rebuild a fresh tree for comparison
    let mut current_items: std::collections::BTreeSet<[u8; 32]> = std::collections::BTreeSet::new();

    for (i, op) in input.ops.iter().enumerate() {
        match op {
            Op::Add(idx) => {
                current_items.insert(*idx);
                map.add_item(*idx, prefix, idx.to_vec());
            }
            Op::Remove(idx) => {
                if current_items.remove(idx) {
                    assert!(map.remove_item(idx));
                }
            }
        }

        // After each mutation, verify the cached hash equals a freshly-built tree's hash
        let incremental_hash = map.hash();

        let mut fresh = ShaMap::new();
        for idx in &current_items {
            fresh.add_item(*idx, prefix, idx.to_vec());
        }
        let fresh_hash = fresh.hash();

        assert_eq!(
            incremental_hash, fresh_hash,
            "incremental tree hash diverged from fresh-build hash after op {} ({} items)",
            i + 1,
            current_items.len()
        );

        assert_eq!(map.len(), current_items.len());
    }
});
