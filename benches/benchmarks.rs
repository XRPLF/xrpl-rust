use criterion::{black_box, criterion_group, criterion_main, Criterion};
use xrpl::core::binarycodec::definitions::get_field_type_name;
use xrpl::core::shamap::ShaMap;
use xrpl::utils::xrp_to_drops;

pub fn bench_xrp_to_drops(c: &mut Criterion) {
    c.bench_function("utils::xrp_to_drops", |b| {
        b.iter(|| xrp_to_drops(black_box("100.000001")))
    });
}

pub fn bench_get_field_type_name(c: &mut Criterion) {
    c.bench_function("core::definitions::definitions::get_field_type_name", |b| {
        b.iter(|| get_field_type_name(black_box("HighLimit")))
    });
}

fn make_shamap_items(count: usize) -> Vec<([u8; 32], Vec<u8>)> {
    (0..count)
        .map(|i| {
            let mut idx = [0u8; 32];
            // Spread items across the index space using simple deterministic mixing
            let bytes = (i as u64).to_be_bytes();
            idx[0] = bytes[7].wrapping_mul(17).wrapping_add(bytes[6]);
            idx[1] = bytes[7].wrapping_mul(37);
            idx[2] = bytes[7].wrapping_mul(53).wrapping_add(bytes[5]);
            idx[3] = bytes[7].wrapping_mul(71);
            // Use remaining bytes for deeper tree diversity
            idx[4..12].copy_from_slice(&bytes);
            let data = idx.to_vec();
            (idx, data)
        })
        .collect()
}

pub fn bench_shamap_hash_100(c: &mut Criterion) {
    let items = make_shamap_items(100);
    let prefix = [0x53, 0x4E, 0x44, 0x00];
    c.bench_function("shamap::hash_100_items", |b| {
        b.iter(|| {
            let mut map = ShaMap::new();
            for (idx, data) in &items {
                map.add_item(*idx, prefix, data.clone());
            }
            black_box(map.hash())
        })
    });
}

pub fn bench_shamap_hash_1000(c: &mut Criterion) {
    let items = make_shamap_items(1000);
    let prefix = [0x53, 0x4E, 0x44, 0x00];
    c.bench_function("shamap::hash_1000_items", |b| {
        b.iter(|| {
            let mut map = ShaMap::new();
            for (idx, data) in &items {
                map.add_item(*idx, prefix, data.clone());
            }
            black_box(map.hash())
        })
    });
}

pub fn bench_shamap_hash_10000(c: &mut Criterion) {
    let items = make_shamap_items(10000);
    let prefix = [0x53, 0x4E, 0x44, 0x00];
    c.bench_function("shamap::hash_10000_items", |b| {
        b.iter(|| {
            let mut map = ShaMap::new();
            for (idx, data) in &items {
                map.add_item(*idx, prefix, data.clone());
            }
            black_box(map.hash())
        })
    });
}

criterion_group!(
    benches,
    bench_xrp_to_drops,
    bench_get_field_type_name,
    bench_shamap_hash_100,
    bench_shamap_hash_1000,
    bench_shamap_hash_10000,
);
criterion_main!(benches);
