//! Fuzz target: ShaMap root hash must be independent of insertion order.
//!
//! The fuzzer provides arbitrary items. We insert them in the given order
//! and in reverse order, then assert both trees produce the same root hash.

#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use xrpl::core::shamap::ShaMap;

#[derive(Arbitrary, Debug)]
struct FuzzInput {
    items: Vec<FuzzItem>,
}

#[derive(Arbitrary, Debug, Clone)]
struct FuzzItem {
    index: [u8; 32],
    data: Vec<u8>,
}

fuzz_target!(|input: FuzzInput| {
    if input.items.is_empty() || input.items.len() > 500 {
        return;
    }

    let prefix = [0x53, 0x4E, 0x44, 0x00];

    // Deduplicate by index (ShaMap doesn't handle duplicate keys)
    let mut seen = std::collections::HashSet::new();
    let items: Vec<&FuzzItem> = input
        .items
        .iter()
        .filter(|item| seen.insert(item.index))
        .collect();

    if items.is_empty() {
        return;
    }

    // Forward order
    let mut map1 = ShaMap::new();
    for item in &items {
        map1.add_item(item.index, prefix, item.data.clone());
    }
    let hash1 = map1.hash();

    // Reverse order
    let mut map2 = ShaMap::new();
    for item in items.iter().rev() {
        map2.add_item(item.index, prefix, item.data.clone());
    }
    let hash2 = map2.hash();

    assert_eq!(hash1, hash2, "root hash must be insertion-order independent");
});
