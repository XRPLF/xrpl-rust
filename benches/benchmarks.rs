use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use xrpl::core::binarycodec::definitions::get_field_type_name;
use xrpl::core::shamap::{verify_proof, ShaMap};
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
            let bytes = (i as u64).to_be_bytes();
            idx[0] = bytes[7].wrapping_mul(17).wrapping_add(bytes[6]);
            idx[1] = bytes[7].wrapping_mul(37);
            idx[2] = bytes[7].wrapping_mul(53).wrapping_add(bytes[5]);
            idx[3] = bytes[7].wrapping_mul(71);
            idx[4..12].copy_from_slice(&bytes);
            let data = idx.to_vec();
            (idx, data)
        })
        .collect()
}

// --- Build + hash (cold) ---

pub fn bench_shamap_build_and_hash(c: &mut Criterion) {
    let mut group = c.benchmark_group("shamap_build_hash");
    let prefix = [0x53, 0x4E, 0x44, 0x00];

    for size in [100, 1_000, 10_000] {
        let items = make_shamap_items(size);
        group.bench_with_input(BenchmarkId::new("items", size), &items, |b, items| {
            b.iter(|| {
                let mut map = ShaMap::new();
                for (idx, data) in items {
                    map.add_item(*idx, prefix, data.clone());
                }
                black_box(map.hash())
            })
        });
    }
    group.finish();
}

// --- Cached hash (hot) ---

pub fn bench_shamap_cached_hash(c: &mut Criterion) {
    let mut group = c.benchmark_group("shamap_cached_hash");
    let prefix = [0x53, 0x4E, 0x44, 0x00];

    for size in [100, 1_000, 10_000] {
        let items = make_shamap_items(size);
        let mut map = ShaMap::new();
        for (idx, data) in &items {
            map.add_item(*idx, prefix, data.clone());
        }
        // Prime the cache
        let _ = map.hash();

        group.bench_with_input(BenchmarkId::new("items", size), &(), |b, _| {
            b.iter(|| black_box(map.hash()))
        });
    }
    group.finish();
}

// --- Incremental add + rehash ---

pub fn bench_shamap_incremental_add(c: &mut Criterion) {
    let mut group = c.benchmark_group("shamap_incremental_add");
    let prefix = [0x53, 0x4E, 0x44, 0x00];

    for base_size in [100, 1_000, 10_000] {
        let items = make_shamap_items(base_size + 1);
        let (base_items, extra) = items.split_at(base_size);

        // Pre-build and hash the base tree
        let mut map = ShaMap::new();
        for (idx, data) in base_items {
            map.add_item(*idx, prefix, data.clone());
        }
        let _ = map.hash();

        let (extra_idx, extra_data) = &extra[0];

        group.bench_with_input(BenchmarkId::new("base_size", base_size), &(), |b, _| {
            b.iter(|| {
                // Add one item and rehash (only dirty path recomputed)
                map.add_item(*extra_idx, prefix, extra_data.clone());
                let h = black_box(map.hash());
                // Remove to reset for next iteration
                map.remove_item(extra_idx);
                let _ = map.hash(); // re-cache
                h
            })
        });
    }
    group.finish();
}

// --- Proof extraction ---

pub fn bench_shamap_proof_extraction(c: &mut Criterion) {
    let mut group = c.benchmark_group("shamap_proof_extract");
    let prefix = [0x53, 0x4E, 0x44, 0x00];

    for size in [100, 1_000, 10_000] {
        let items = make_shamap_items(size);
        let mut map = ShaMap::new();
        for (idx, data) in &items {
            map.add_item(*idx, prefix, data.clone());
        }
        let _ = map.hash(); // prime cache

        let target = items[size / 2].0;

        group.bench_with_input(BenchmarkId::new("items", size), &(), |b, _| {
            b.iter(|| black_box(map.extract_proof(&target)))
        });
    }
    group.finish();
}

// --- Proof verification ---

pub fn bench_shamap_proof_verify(c: &mut Criterion) {
    let mut group = c.benchmark_group("shamap_proof_verify");
    let prefix = [0x53, 0x4E, 0x44, 0x00];

    for size in [100, 1_000, 10_000] {
        let items = make_shamap_items(size);
        let mut map = ShaMap::new();
        for (idx, data) in &items {
            map.add_item(*idx, prefix, data.clone());
        }
        let root_hash = map.hash();
        let target = items[size / 2].0;
        let proof = map.extract_proof(&target).unwrap();

        group.bench_with_input(BenchmarkId::new("items", size), &(), |b, _| {
            b.iter(|| black_box(verify_proof(&proof, &root_hash)))
        });
    }
    group.finish();
}

// --- Lookup ---

pub fn bench_shamap_contains(c: &mut Criterion) {
    let mut group = c.benchmark_group("shamap_contains");
    let prefix = [0x53, 0x4E, 0x44, 0x00];

    for size in [100, 1_000, 10_000] {
        let items = make_shamap_items(size);
        let mut map = ShaMap::new();
        for (idx, data) in &items {
            map.add_item(*idx, prefix, data.clone());
        }

        let target = items[size / 2].0;

        group.bench_with_input(BenchmarkId::new("items", size), &(), |b, _| {
            b.iter(|| black_box(map.contains(&target)))
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_xrp_to_drops,
    bench_get_field_type_name,
    bench_shamap_build_and_hash,
    bench_shamap_cached_hash,
    bench_shamap_incremental_add,
    bench_shamap_proof_extraction,
    bench_shamap_proof_verify,
    bench_shamap_contains,
);
criterion_main!(benches);
