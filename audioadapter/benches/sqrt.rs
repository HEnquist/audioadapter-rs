use audioadapter::stats::sqrt_newton;
use criterion::{Criterion, criterion_group, criterion_main};
use num_traits::Float;
use std::hint::black_box;

fn bench_sqrt(c: &mut Criterion) {
    let values: Vec<f64> = (1..=2048).map(|index| (index as f64) * 0.125).collect();

    c.bench_function("sqrt_newton", |b| {
        b.iter(|| {
            let mut sum = 0.0;
            for &value in &values {
                sum += black_box(sqrt_newton(black_box(value)));
            }
            black_box(sum)
        })
    });

    c.bench_function("sqrt_float_trait", |b| {
        b.iter(|| {
            let mut sum = 0.0;
            for &value in &values {
                sum += black_box(<f64 as Float>::sqrt(black_box(value)));
            }
            black_box(sum)
        })
    });
}

criterion_group!(benches, bench_sqrt);
criterion_main!(benches);
