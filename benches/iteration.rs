use audioadapter::sample::*;
use audioadapter::*;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

// plain nested loops with unsafe read calls
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

// plain nested loops with safe read calls
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

// use the iterators
fn iter_with_iter_trait(buf: &direct::SequentialSliceOfVecs<&[Vec<i32>]>) -> i32 {
    let mut sum = 0;
    for channel in buf.iter_channels() {
        for value in channel {
            sum += value;
        }
    }
    return sum;
}

pub fn bench_with_iter_trait(c: &mut Criterion) {
    let data = vec![vec![1_i32; 10000], vec![2_i32; 10000]];
    let buffer = direct::SequentialSliceOfVecs::new(&data, 2, 10000).unwrap();
    c.bench_function("iterators", |b| {
        b.iter(|| black_box(iter_with_iter_trait(black_box(&buffer))))
    });
}

// use the iterators with format conversion
fn iter_with_i32le_float_conversion(
    buf: &number_to_float::SequentialNumbers<&[I32LE], f32>,
) -> f32 {
    let mut sum = 0.0;
    for channel in buf.iter_channels() {
        for value in channel {
            sum += value;
        }
    }
    return sum;
}

pub fn bench_with_i32le_float_conversion(c: &mut Criterion) {
    let data = vec![1_u8; 80000];
    let buffer =
        number_to_float::SequentialNumbers::<&[I32LE], f32>::new_from_bytes(&data, 2, 10000)
            .unwrap();
    c.bench_function("convert_i32le_to_float", |b| {
        b.iter(|| black_box(iter_with_i32le_float_conversion(black_box(&buffer))))
    });
}

// use the iterators with format conversion
fn iter_with_i24le_float_conversion(
    buf: &number_to_float::SequentialNumbers<&[I24LE<3>], f32>,
) -> f32 {
    let mut sum = 0.0;
    for channel in buf.iter_channels() {
        for value in channel {
            sum += value;
        }
    }
    return sum;
}

pub fn bench_with_i24le_float_conversion(c: &mut Criterion) {
    let data = vec![1_u8; 60000];
    let buffer =
        number_to_float::SequentialNumbers::<&[I24LE<3>], f32>::new_from_bytes(&data, 2, 10000)
            .unwrap();
    c.bench_function("convert_i24le_to_float", |b| {
        b.iter(|| black_box(iter_with_i24le_float_conversion(black_box(&buffer))))
    });
}

// standard iteration of slices, for comparison
fn iter_slice(buf: &[Vec<i32>]) -> i32 {
    let sum = buf.iter().map(|v| v.iter().sum::<i32>()).sum();
    return sum;
}

pub fn bench_slice_iter(c: &mut Criterion) {
    let data = vec![vec![1_i32; 10000], vec![2_i32; 10000]];
    c.bench_function("slice_iter", |b| {
        b.iter(|| black_box(iter_slice(black_box(&data))))
    });
}

criterion_group!(
    benches,
    bench_with_unchecked_loop,
    bench_with_safe_loop,
    bench_with_iter_trait,
    bench_slice_iter,
    bench_with_i32le_float_conversion,
    bench_with_i24le_float_conversion
);
criterion_main!(benches);
