use criterion::{black_box, criterion_group, criterion_main, Criterion};
use xrpl::core::binarycodec::definitions::get_field_type_name;
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

criterion_group!(benches, bench_xrp_to_drops, bench_get_field_type_name);
criterion_main!(benches);
