//! # [ndarray](https://crates.io/crates/ndarray) crate compatibility
//!
//! This module implements the `audioadapter` traits for two-dimensional
//! [ndarray](https://crates.io/crates/ndarray) arrays (`ArrayBase<_, Ix2>`,
//! which includes `Array2<T>` and array views).
//!
//! Because both channel-major and frame-major storage are common, the axis
//! order is selected explicitly when the adapter is created:
//!
//! * [`NdarrayAdapter::new_channels_frames`] for arrays shaped `(channels, frames)`.
//! * [`NdarrayAdapter::new_frames_channels`] for arrays shaped `(frames, channels)`.
//!
//! Sample access uses ndarray indexing, so it is correct for any memory layout.
//! The bulk copy helpers take a fast path that copies directly from a contiguous
//! slice when the relevant axis is contiguous (the common standard-layout case),
//! and fall back to element-wise access otherwise.
//!
//! ```
//! use ndarray::array;
//! use audioadapter::Adapter;
//! use audioadapter_compat_ndarray::NdarrayAdapter;
//!
//! // Two channels, three frames, stored channel-major.
//! let data = array![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]];
//! let adapter = NdarrayAdapter::new_channels_frames(data.view());
//! assert_eq!(adapter.channels(), 2);
//! assert_eq!(adapter.frames(), 3);
//! assert_eq!(adapter.read_sample(1, 2), Some(6.0));
//! ```

use core::marker::PhantomData;

use ndarray::{ArrayBase, Axis, Data, DataMut, Ix2};

use audioadapter::{Adapter, AdapterMut};

mod sealed {
    pub trait Sealed {}
}

/// Axis-order marker describing how the two array axes map to channels and frames.
pub trait AxisOrder: sealed::Sealed {
    /// The array axis (0 or 1) along which the channel index runs.
    const CHANNEL_AXIS: usize;
    /// Number of channels for an array of the given shape.
    fn channels(dim: (usize, usize)) -> usize;
    /// Number of frames for an array of the given shape.
    fn frames(dim: (usize, usize)) -> usize;
    /// Map a `(channel, frame)` pair to an ndarray `(row, column)` index.
    fn index(channel: usize, frame: usize) -> (usize, usize);
}

/// Axis order for arrays shaped `(channels, frames)` (channel-major).
pub struct ChannelsFrames;
impl sealed::Sealed for ChannelsFrames {}
impl AxisOrder for ChannelsFrames {
    const CHANNEL_AXIS: usize = 0;
    fn channels(dim: (usize, usize)) -> usize {
        dim.0
    }
    fn frames(dim: (usize, usize)) -> usize {
        dim.1
    }
    fn index(channel: usize, frame: usize) -> (usize, usize) {
        (channel, frame)
    }
}

/// Axis order for arrays shaped `(frames, channels)` (frame-major).
pub struct FramesChannels;
impl sealed::Sealed for FramesChannels {}
impl AxisOrder for FramesChannels {
    const CHANNEL_AXIS: usize = 1;
    fn channels(dim: (usize, usize)) -> usize {
        dim.1
    }
    fn frames(dim: (usize, usize)) -> usize {
        dim.0
    }
    fn index(channel: usize, frame: usize) -> (usize, usize) {
        (frame, channel)
    }
}

/// A wrapper implementing the `audioadapter` traits for a two-dimensional
/// ndarray array.
///
/// The axis order `O` records whether the array is channel-major
/// ([`ChannelsFrames`]) or frame-major ([`FramesChannels`]).
pub struct NdarrayAdapter<U, O> {
    array: U,
    _order: PhantomData<O>,
}

impl<S> NdarrayAdapter<ArrayBase<S, Ix2>, ChannelsFrames>
where
    S: Data,
{
    /// Wrap an array shaped `(channels, frames)`.
    pub fn new_channels_frames(array: ArrayBase<S, Ix2>) -> Self {
        Self {
            array,
            _order: PhantomData,
        }
    }
}

impl<S> NdarrayAdapter<ArrayBase<S, Ix2>, FramesChannels>
where
    S: Data,
{
    /// Wrap an array shaped `(frames, channels)`.
    pub fn new_frames_channels(array: ArrayBase<S, Ix2>) -> Self {
        Self {
            array,
            _order: PhantomData,
        }
    }
}

impl<U, O> NdarrayAdapter<U, O> {
    /// Consume the adapter and return the wrapped array.
    pub fn into_inner(self) -> U {
        self.array
    }

    /// Get a reference to the wrapped array.
    pub fn inner(&self) -> &U {
        &self.array
    }
}

unsafe impl<S, O> Adapter<S::Elem> for NdarrayAdapter<ArrayBase<S, Ix2>, O>
where
    S: Data,
    S::Elem: Clone,
    O: AxisOrder,
{
    fn channels(&self) -> usize {
        O::channels(self.array.dim())
    }

    fn frames(&self) -> usize {
        O::frames(self.array.dim())
    }

    unsafe fn read_sample_unchecked(&self, channel: usize, frame: usize) -> S::Elem {
        unsafe { self.array.uget(O::index(channel, frame)) }.clone()
    }

    fn copy_from_channel_to_slice(
        &self,
        channel: usize,
        skip: usize,
        slice: &mut [S::Elem],
    ) -> usize {
        if channel >= self.channels() || skip >= self.frames() {
            return 0;
        }
        let view = self.array.index_axis(Axis(O::CHANNEL_AXIS), channel);
        let available = view.len() - skip;
        let to_copy = available.min(slice.len());
        if let Some(contiguous) = view.as_slice() {
            slice[..to_copy].clone_from_slice(&contiguous[skip..skip + to_copy]);
        } else {
            for (out, sample) in slice.iter_mut().zip(view.iter().skip(skip)).take(to_copy) {
                *out = sample.clone();
            }
        }
        to_copy
    }
}

unsafe impl<S, O> AdapterMut<S::Elem> for NdarrayAdapter<ArrayBase<S, Ix2>, O>
where
    S: DataMut,
    S::Elem: Clone,
    O: AxisOrder,
{
    unsafe fn write_sample_unchecked(
        &mut self,
        channel: usize,
        frame: usize,
        value: &S::Elem,
    ) -> bool {
        unsafe { *self.array.uget_mut(O::index(channel, frame)) = value.clone() };
        false
    }

    fn copy_from_slice_to_channel(
        &mut self,
        channel: usize,
        skip: usize,
        slice: &[S::Elem],
    ) -> (usize, usize) {
        if channel >= Adapter::channels(self) || skip >= Adapter::frames(self) {
            return (0, 0);
        }
        let mut view = self.array.index_axis_mut(Axis(O::CHANNEL_AXIS), channel);
        let available = view.len() - skip;
        let to_copy = available.min(slice.len());
        if let Some(contiguous) = view.as_slice_mut() {
            contiguous[skip..skip + to_copy].clone_from_slice(&slice[..to_copy]);
        } else {
            for (dest, sample) in view.iter_mut().skip(skip).zip(slice.iter()).take(to_copy) {
                *dest = sample.clone();
            }
        }
        (to_copy, 0)
    }
}

//   _____         _
//  |_   _|__  ___| |_ ___
//    | |/ _ \/ __| __/ __|
//    | |  __/\__ \ |_\__ \
//    |_|\___||___/\__|___/

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::array;

    #[test]
    fn channels_frames_dimensions_and_read() {
        let data = array![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]];
        let adapter = NdarrayAdapter::new_channels_frames(data.view());
        assert_eq!(adapter.channels(), 2);
        assert_eq!(adapter.frames(), 3);
        assert_eq!(adapter.read_sample(0, 0), Some(1.0));
        assert_eq!(adapter.read_sample(1, 2), Some(6.0));
        assert_eq!(adapter.read_sample(2, 0), None);
        assert_eq!(adapter.read_sample(0, 3), None);
    }

    #[test]
    fn frames_channels_dimensions_and_read() {
        // Same logical audio, stored frame-major (each row is a frame).
        let data = array![[1.0, 4.0], [2.0, 5.0], [3.0, 6.0]];
        let adapter = NdarrayAdapter::new_frames_channels(data.view());
        assert_eq!(adapter.channels(), 2);
        assert_eq!(adapter.frames(), 3);
        assert_eq!(adapter.read_sample(0, 0), Some(1.0));
        assert_eq!(adapter.read_sample(1, 2), Some(6.0));
    }

    #[test]
    fn copy_channel_to_slice_contiguous() {
        // Channel-major standard layout: each channel is a contiguous row.
        let data = array![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]];
        let adapter = NdarrayAdapter::new_channels_frames(data.view());
        let mut out = [0.0; 2];
        let copied = adapter.copy_from_channel_to_slice(1, 1, &mut out);
        assert_eq!(copied, 2);
        assert_eq!(out, [5.0, 6.0]);
    }

    #[test]
    fn copy_channel_to_slice_strided() {
        // Frame-major: a channel runs down a column, which is strided.
        let data = array![[1.0, 4.0], [2.0, 5.0], [3.0, 6.0]];
        let adapter = NdarrayAdapter::new_frames_channels(data.view());
        let mut out = [0.0; 3];
        let copied = adapter.copy_from_channel_to_slice(1, 0, &mut out);
        assert_eq!(copied, 3);
        assert_eq!(out, [4.0, 5.0, 6.0]);
    }

    #[test]
    fn write_and_copy_from_slice() {
        let mut data = array![[0.0, 0.0, 0.0], [0.0, 0.0, 0.0]];
        let mut adapter = NdarrayAdapter::new_channels_frames(data.view_mut());
        assert_eq!(adapter.write_sample(0, 1, &9.0), Some(false));
        assert_eq!(adapter.read_sample(0, 1), Some(9.0));
        let (copied, clipped) = adapter.copy_from_slice_to_channel(1, 0, &[7.0, 8.0]);
        assert_eq!((copied, clipped), (2, 0));
        assert_eq!(adapter.read_sample(1, 0), Some(7.0));
        assert_eq!(adapter.read_sample(1, 1), Some(8.0));
        assert_eq!(adapter.read_sample(1, 2), Some(0.0));
    }
}
