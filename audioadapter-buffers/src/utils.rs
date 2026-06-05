//! A collection of utilities for working with adapters.
//!
//! This module gathers small helpers that build on the adapter traits. For now
//! that is just the slice-copy functions described below; more may be added
//! over time.
//!
//! # Copying between an adapter and a plain slice
//!
//! The `to_interleaved`, `to_sequential`, `from_interleaved`, and
//! `from_sequential` functions bridge the common case where a caller holds a
//! raw `&[T]` or `&mut [T]` in a known memory layout (interleaved or sequential)
//! and wants to copy it to or from an `Adapter`/`AdapterMut`, without wrapping
//! the slice by hand. Only the slice side is a fixed layout; the other side is
//! an arbitrary adapter, so it can be any layout or sample format, including the
//! converting wrappers in [`crate::adapter_to_float`] and the sparse buffers in
//! [`crate::direct`].
//!
//! Each helper works like
//! [`AdapterMut::copy_from_other`](audioadapter::AdapterMut::copy_from_other):
//! it takes a `src_skip`, a `dst_skip`, and a `take`, copies `take` frames for
//! all channels, and returns the number of samples clipped during conversion,
//! or `None` if the requested region does not fit in either the adapter or the
//! slice. The slice carries its own frame count (`slice.len() / channels`); a
//! slice longer than a whole number of frames keeps the convention of the
//! buffer wrappers, with the trailing partial frame left unused.
//!
//! Note that a same-type interleaved-to-sequential copy of two plain slices is
//! just a transpose and does not need an adapter at all. These helpers earn
//! their place only because the adapter side is unconstrained: it may relayout,
//! convert the sample format, or be sparse.
//!
//! ## Sparse buffers
//!
//! The sparse buffers in [`crate::direct`] report their full channel count but
//! only store some channels. They compose cleanly here:
//!
//! - As a source, absent channels read back as `T::default()`, so
//!   `to_interleaved` and `to_sequential` write zeros into the output slice
//!   for those channels.
//! - As a destination, writes to absent channels are dropped, so
//!   `from_interleaved` and `from_sequential` simply skip the samples that
//!   have nowhere to go.
//!
//! Both directions stay in bounds and never panic.

use audioadapter::{Adapter, AdapterMut};

use crate::direct::{InterleavedSlice, SequentialSlice};

/// Number of whole frames a flat slice of `len` samples holds for `channels`
/// channels. Zero channels yields zero frames rather than dividing by zero.
fn slice_frames(len: usize, channels: usize) -> usize {
    // `checked_div` yields `None` for zero channels, which we treat as zero frames.
    len.checked_div(channels).unwrap_or(0)
}

/// Copy `take` frames from `src` into `out`, laid out interleaved.
///
/// `src_skip` and `dst_skip` are the starting frames in the adapter and the
/// slice. Returns the number of samples clipped during conversion, or `None` if
/// the region does not fit in either side. See the [module docs](self).
pub fn to_interleaved<T: Clone>(
    src: &dyn Adapter<T>,
    out: &mut [T],
    src_skip: usize,
    dst_skip: usize,
    take: usize,
) -> Option<usize> {
    let channels = src.channels();
    let frames = slice_frames(out.len(), channels);
    // `frames` is `out.len() / channels`, so `out` is long enough by construction.
    let mut dst =
        InterleavedSlice::new_mut(out, channels, frames).expect("out fits channels*frames");
    dst.copy_from_other(src, src_skip, dst_skip, take)
}

/// Copy `take` frames from `src` into `out`, laid out sequential (channel by
/// channel).
///
/// `src_skip` and `dst_skip` are the starting frames in the adapter and the
/// slice. Returns the number of samples clipped during conversion, or `None` if
/// the region does not fit in either side. See the [module docs](self).
pub fn to_sequential<T: Clone>(
    src: &dyn Adapter<T>,
    out: &mut [T],
    src_skip: usize,
    dst_skip: usize,
    take: usize,
) -> Option<usize> {
    let channels = src.channels();
    let frames = slice_frames(out.len(), channels);
    let mut dst =
        SequentialSlice::new_mut(out, channels, frames).expect("out fits channels*frames");
    dst.copy_from_other(src, src_skip, dst_skip, take)
}

/// Copy `take` frames from the interleaved slice `input` into the adapter `dst`.
///
/// `src_skip` and `dst_skip` are the starting frames in the slice and the
/// adapter. Returns the number of samples clipped during conversion, or `None`
/// if the region does not fit in either side. See the [module docs](self).
pub fn from_interleaved<T: Clone>(
    input: &[T],
    dst: &mut dyn AdapterMut<T>,
    src_skip: usize,
    dst_skip: usize,
    take: usize,
) -> Option<usize> {
    let channels = dst.channels();
    let frames = slice_frames(input.len(), channels);
    let src = InterleavedSlice::new(input, channels, frames).expect("input fits channels*frames");
    dst.copy_from_other(&src, src_skip, dst_skip, take)
}

/// Copy `take` frames from the sequential slice `input` (channel by channel)
/// into the adapter `dst`.
///
/// `src_skip` and `dst_skip` are the starting frames in the slice and the
/// adapter. Returns the number of samples clipped during conversion, or `None`
/// if the region does not fit in either side. See the [module docs](self).
pub fn from_sequential<T: Clone>(
    input: &[T],
    dst: &mut dyn AdapterMut<T>,
    src_skip: usize,
    dst_skip: usize,
    take: usize,
) -> Option<usize> {
    let channels = dst.channels();
    let frames = slice_frames(input.len(), channels);
    let src = SequentialSlice::new(input, channels, frames).expect("input fits channels*frames");
    dst.copy_from_other(&src, src_skip, dst_skip, take)
}

//   _____         _
//  |_   _|__  ___| |_ ___
//    | |/ _ \/ __| __/ __|
//    | |  __/\__ \ |_\__ \
//    |_|\___||___/\__|___/

#[cfg(test)]
mod tests {
    use super::*;
    use crate::direct::{InterleavedSlice, SequentialSlice};

    // sequential [ch0: 1,2,3][ch1: 4,5,6]  <->  interleaved [1,4,2,5,3,6]

    #[test]
    fn adapter_to_interleaved_whole() {
        let data = [1, 2, 3, 4, 5, 6];
        let src = SequentialSlice::new(&data, 2, 3).unwrap();
        let mut out = [0; 6];
        let clipped = to_interleaved(&src, &mut out, 0, 0, 3);
        assert_eq!(clipped, Some(0));
        assert_eq!(out, [1, 4, 2, 5, 3, 6]);
    }

    #[test]
    fn adapter_to_sequential_whole() {
        let data = [1, 4, 2, 5, 3, 6];
        let src = InterleavedSlice::new(&data, 2, 3).unwrap();
        let mut out = [0; 6];
        assert_eq!(to_sequential(&src, &mut out, 0, 0, 3), Some(0));
        assert_eq!(out, [1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn interleaved_into_adapter_whole() {
        let input = [1, 4, 2, 5, 3, 6];
        let mut store = [0; 6];
        let mut dst = SequentialSlice::new_mut(&mut store, 2, 3).unwrap();
        assert_eq!(from_interleaved(&input, &mut dst, 0, 0, 3), Some(0));
        assert_eq!(store, [1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn sequential_into_adapter_whole() {
        let input = [1, 2, 3, 4, 5, 6];
        let mut store = [0; 6];
        let mut dst = InterleavedSlice::new_mut(&mut store, 2, 3).unwrap();
        assert_eq!(from_sequential(&input, &mut dst, 0, 0, 3), Some(0));
        assert_eq!(store, [1, 4, 2, 5, 3, 6]);
    }

    #[test]
    fn partial_take_copies_a_prefix() {
        // Source has 3 frames, copy only the first 2 into a 2-frame slice.
        let data = [1, 2, 3, 4, 5, 6]; // ch0 = [1,2,3], ch1 = [4,5,6]
        let src = SequentialSlice::new(&data, 2, 3).unwrap();
        let mut out = [0; 4];
        assert_eq!(to_interleaved(&src, &mut out, 0, 0, 2), Some(0));
        assert_eq!(out, [1, 4, 2, 5]);
    }

    #[test]
    fn src_skip_reads_from_offset() {
        let data = [1, 2, 3, 4, 5, 6]; // ch0 = [1,2,3], ch1 = [4,5,6]
        let src = SequentialSlice::new(&data, 2, 3).unwrap();
        let mut out = [0; 4];
        // Skip the first source frame, take the next two: frames 1 and 2.
        assert_eq!(to_interleaved(&src, &mut out, 1, 0, 2), Some(0));
        assert_eq!(out, [2, 5, 3, 6]);
    }

    #[test]
    fn dst_skip_appends() {
        // Write one frame at frame 2 of a 3-frame adapter.
        let mut store = [0; 6];
        let mut dst = SequentialSlice::new_mut(&mut store, 2, 3).unwrap();
        let input = [7, 70]; // one interleaved frame: ch0 = 7, ch1 = 70
        assert_eq!(from_interleaved(&input, &mut dst, 0, 2, 1), Some(0));
        // ch0 = [0,0,7], ch1 = [0,0,70]
        assert_eq!(store, [0, 0, 7, 0, 0, 70]);
    }

    #[test]
    fn region_that_does_not_fit_returns_none() {
        let data = [1, 2, 3, 4, 5, 6];
        let src = SequentialSlice::new(&data, 2, 3).unwrap();
        let mut out = [0; 4]; // room for 2 frames

        // take larger than the slice can hold
        assert_eq!(to_interleaved(&src, &mut out, 0, 0, 3), None);
        // take larger than the source has
        let mut big = [0; 12]; // room for 6 frames
        assert_eq!(to_interleaved(&src, &mut big, 0, 0, 4), None);
        // overflowing skip must not wrap
        assert_eq!(to_interleaved(&src, &mut out, usize::MAX, 0, 2), None);
    }

    #[test]
    fn longer_slice_ignores_trailing_partial_frame() {
        let data = [1, 2, 3, 4, 5, 6];
        let src = SequentialSlice::new(&data, 2, 3).unwrap();
        let mut ragged = [0; 5]; // holds 2 whole frames, 1 trailing sample unused
        assert_eq!(to_interleaved(&src, &mut ragged, 0, 0, 2), Some(0));
        assert_eq!(ragged, [1, 4, 2, 5, 0]); // last element untouched
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn sparse_source_reads_zeros_for_absent_channels() {
        use crate::direct::SparseSequentialSliceOfVecs;
        extern crate alloc;
        use alloc::vec;

        // 2 channels, 3 frames, but only channel 0 is present.
        let data = vec![vec![1, 2, 3], vec![]];
        let mask = [true, false];
        let src = SparseSequentialSliceOfVecs::new(&data, 2, 3, &mask).unwrap();

        let mut out = [9; 6];
        assert_eq!(to_interleaved(&src, &mut out, 0, 0, 3), Some(0));
        // ch1 reads as default (0): frames [ch0, ch1] = [1,0],[2,0],[3,0]
        assert_eq!(out, [1, 0, 2, 0, 3, 0]);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn sparse_destination_drops_absent_channels() {
        use crate::direct::SparseSequentialSliceOfVecs;
        extern crate alloc;
        use alloc::vec;

        // 2 channels, 3 frames, only channel 0 is writable.
        let mut data = vec![vec![0, 0, 0], vec![0, 0, 0]];
        let mask = [true, false];
        let mut dst = SparseSequentialSliceOfVecs::new_mut(&mut data, 2, 3, &mask).unwrap();

        // interleaved input: ch0 = [1,2,3], ch1 = [10,20,30]
        let input = [1, 10, 2, 20, 3, 30];
        assert_eq!(from_interleaved(&input, &mut dst, 0, 0, 3), Some(0));
        // ch0 was written, ch1 writes were dropped (vector left untouched).
        assert_eq!(data[0], [1, 2, 3]);
        assert_eq!(data[1], [0, 0, 0]);
    }

    #[test]
    fn from_interleaved_reports_clipping() {
        use crate::adapter_to_float::ConvertNumbers;
        use audioadapter::AdapterMut;

        // A float-to-i16 converting destination: values outside [-1, 1] clip.
        let mut store = [0i16; 2];
        {
            let mut inner = InterleavedSlice::new_mut(&mut store, 1, 2).unwrap();
            let mut dst: ConvertNumbers<&mut dyn AdapterMut<i16>, f32> =
                ConvertNumbers::new_mut(&mut inner as &mut dyn AdapterMut<i16>);
            // Both samples are well above full scale, so both clip on the way in.
            let input = [2.0_f32, 2.0];
            assert_eq!(from_interleaved(&input, &mut dst, 0, 0, 2), Some(2));
        }
        // Both samples saturated to the i16 maximum.
        assert_eq!(store, [i16::MAX, i16::MAX]);
    }
}
