use audioadapter::*;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn iter_with_unchecked_loop(buf: &direct::SequentialSliceOfVecs<&[Vec<i32>]>) -> i32 {
    let mut sum = 0;
    unsafe {
        for channel in 0..buf.channels() {
            for frame in 0..buf.frames() {
                sum += buf.read_sample_unchecked(channel, frame);
            }
        }
    }
    return sum;
}

pub fn bench_with_unchecked_loop(c: &mut Criterion) {
    let data = vec![vec![1_i32; 10000], vec![2_i32; 10000]];
    let buffer = direct::SequentialSliceOfVecs::new(&data, 2, 10000).unwrap();
    c.bench_function("loop", |b| {
        b.iter(|| black_box(iter_with_unchecked_loop(black_box(&buffer))))
    });
}

fn iter_with_safe_loop(buf: &direct::SequentialSliceOfVecs<&[Vec<i32>]>) -> i32 {
    let mut sum = 0;
    for channel in 0..buf.channels() {
        for frame in 0..buf.frames() {
            sum += buf.read_sample(channel, frame).unwrap();
        }
    }
    return sum;
}

pub fn bench_with_safe_loop(c: &mut Criterion) {
    let data = vec![vec![1_i32; 10000], vec![2_i32; 10000]];
    let buffer = direct::SequentialSliceOfVecs::new(&data, 2, 10000).unwrap();
    c.bench_function("safe_loop", |b| {
        b.iter(|| black_box(iter_with_safe_loop(black_box(&buffer))))
    });
}

fn iter_slice(buf: &[Vec<i32>]) -> i32 {
    let sum = buf.iter().map(|v| v.iter().sum::<i32>()).sum();
    return sum;
}

pub fn bench_slice_iter(c: &mut Criterion) {
    let data = vec![vec![1_i32; 10000], vec![2_i32; 10000]];
    c.bench_function("iter_slice", |b| {
        b.iter(|| black_box(iter_slice(black_box(&data))))
    });
}

criterion_group!(
    benches,
    bench_with_unchecked_loop,
    bench_with_safe_loop,
    bench_slice_iter
);
criterion_main!(benches);
