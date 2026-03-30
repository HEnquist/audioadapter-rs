use criterion::{Criterion, criterion_group, criterion_main};
use num_traits::Float;
use std::hint::black_box;

// Duplicated from `audioadapter/src/stats.rs` (`sqrt_newton`),
// since that helper is intentionally not part of the public API.
fn sqrt_newton(value: f64) -> f64 {
    if value.is_nan() {
        return value;
    }
    if value.is_infinite() {
        return if value.is_sign_positive() {
            f64::INFINITY
        } else {
            0.0
        };
    }
    if value <= 0.0 {
        return 0.0;
    }

    let mut estimate = f64::from_bits((value.to_bits() + (1023_u64 << 52)) >> 1);
    for _ in 0..5 {
        estimate = 0.5 * (estimate + value / estimate);
    }
    estimate
}

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
