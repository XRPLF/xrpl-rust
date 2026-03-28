//! Fuzz target: adding then removing items restores the original hash.
//!
//! Start with a base set, hash it. Add extra items, remove them.
//! The hash must return to the original value.

#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use xrpl::core::shamap::ShaMap;

#[derive(Arbitrary, Debug)]
struct FuzzInput {
    base_items: Vec<[u8; 32]>,
    extra_items: Vec<[u8; 32]>,
}

fuzz_target!(|input: FuzzInput| {
    if input.base_items.is_empty() || input.base_items.len() > 200 {
        return;
    }
    if input.extra_items.is_empty() || input.extra_items.len() > 50 {
        return;
    }

    let prefix = [0x4D, 0x4C, 0x4E, 0x00]; // ACCOUNT_STATE_ENTRY

    // Deduplicate base items
    let mut seen = std::collections::HashSet::new();
    let base: Vec<[u8; 32]> = input
        .base_items
        .iter()
        .copied()
        .filter(|idx| seen.insert(*idx))
        .collect();

    // Extra items must not overlap with base
    let extras: Vec<[u8; 32]> = input
        .extra_items
        .iter()
        .copied()
        .filter(|idx| seen.insert(*idx))
        .collect();

    if base.is_empty() || extras.is_empty() {
        return;
    }

    // Build base tree
    let mut map = ShaMap::new();
    for idx in &base {
        map.add_item(*idx, prefix, idx.to_vec());
    }
    let base_hash = map.hash();
    let base_len = map.len();

    // Add extras
    for idx in &extras {
        map.add_item(*idx, prefix, idx.to_vec());
    }
    assert_eq!(map.len(), base_len + extras.len());

    // Remove extras
    for idx in &extras {
        assert!(map.remove_item(idx), "extra item must be removable");
    }
    assert_eq!(map.len(), base_len);

    let roundtrip_hash = map.hash();
    assert_eq!(
        base_hash, roundtrip_hash,
        "hash must be restored after add+remove roundtrip"
    );
});
