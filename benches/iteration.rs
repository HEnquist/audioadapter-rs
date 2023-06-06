use criterion::{black_box, criterion_group, criterion_main, Criterion};
use audiobuffer::*;



fn iter_with_box(buf: &SliceOfChannelVecs<i32>) -> i32 {
    let mut sum = 0;
    for channel in 0..buf.channels() {
        sum += buf.channel(channel).sum::<i32>();
    }
    return sum
}

pub fn bench_with_box(c: &mut Criterion) {
    let mut data = vec![vec![1_i32; 10000], vec![2_i32; 10000]];
    let mut buffer = SliceOfChannelVecs::new(&mut data, 2, 10000).unwrap();
    c.bench_function("box", |b| b.iter(|| black_box(iter_with_box(black_box(&buffer)))));
}



fn iter_with_iterator(buf: &SliceOfChannelVecs<i32>) -> i32 {
    let mut sum = 0;
    for channel in buf.iter_channels() {
        sum += channel.sum::<i32>();
    }
    return sum
}

pub fn bench_with_iterator(c: &mut Criterion) {
    let mut data = vec![vec![1_i32; 10000], vec![2_i32; 10000]];
    let mut buffer = SliceOfChannelVecs::new(&mut data, 2, 10000).unwrap();
    c.bench_function("new_iter", |b| b.iter(|| black_box(iter_with_iterator(black_box(&buffer)))));
}



fn iter_with_loop(buf: &SliceOfChannelVecs<i32>) -> i32 {
    let mut sum = 0;
    unsafe { 
        for channel in 0..buf.channels() {
            for frame in 0..buf.frames() {
                sum += *buf.get_unchecked(channel, frame);
            }
        }
    }
    return sum
}

pub fn bench_with_loop(c: &mut Criterion) {
    let mut data = vec![vec![1_i32; 10000], vec![2_i32; 10000]];
    let mut buffer = SliceOfChannelVecs::new(&mut data, 2, 10000).unwrap();
    c.bench_function("loop", |b| b.iter(|| black_box(iter_with_loop(black_box(&buffer)))));
}



fn iter_with_safe_loop(buf: &SliceOfChannelVecs<i32>) -> i32 {
    let mut sum = 0;
    for channel in 0..buf.channels() {
        for frame in 0..buf.frames() {
            sum += *buf.get(channel, frame).unwrap();
        }
    }
    return sum
}

pub fn bench_with_safe_loop(c: &mut Criterion) {
    let mut data = vec![vec![1_i32; 10000], vec![2_i32; 10000]];
    let mut buffer = SliceOfChannelVecs::new(&mut data, 2, 10000).unwrap();
    c.bench_function("safe_loop", |b| b.iter(|| black_box(iter_with_safe_loop(black_box(&buffer)))));
}



fn iter_slice(buf: &[Vec<i32>]) -> i32 {
    let sum = buf.iter().map(|v| v.iter().sum::<i32>()).sum();
    return sum
}

pub fn bench_slice_iter(c: &mut Criterion) {
    let mut data = vec![vec![1_i32; 10000], vec![2_i32; 10000]];
    c.bench_function("iter_slice", |b| b.iter(|| black_box(iter_slice(black_box(&data)))));
}



criterion_group!(benches, bench_with_box, bench_with_iterator, bench_with_loop, bench_with_safe_loop, bench_slice_iter);
criterion_main!(benches);