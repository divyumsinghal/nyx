use criterion::{black_box, criterion_group, criterion_main, Criterion};
use nyx_tests::security;

fn bench_sqli_payload_scan(c: &mut Criterion) {
    c.bench_function("sqli_payload_scan", |b| {
        b.iter(|| {
            let payloads = security::sql_injection::all_payloads();
            let risky = payloads
                .iter()
                .filter(|p| p.contains("UNION") || p.contains("DROP"))
                .count();
            black_box(risky)
        })
    });
}

criterion_group!(benches, bench_sqli_payload_scan);
criterion_main!(benches);
