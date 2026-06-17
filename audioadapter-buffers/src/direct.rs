//! # Direct wrappers
//!
//! This module is a collection of wrappers that implement the
//! `audioadapter` traits for various data structures.
//!
//! These wrap data structures where
//! the samples are already stored in the desired format.
//!
//! ## Available wrappers
//! Wrappers are available for plain slices, `&[T]`,
//! and slices of vectors, `&[Vec<T>]`.
//!
//! Each wrapper exists in an _interleaved_ and _sequential_ version.
//!
//! ### Example
//! Wrap a Vec of i32 as an interleaved buffer
//! and print all the values.
//! ```
//! use audioadapter::Adapter;
//! use audioadapter_buffers::direct::InterleavedSlice;
//!
//! // make a vector with some dummy data.
//! // 2 channels * 3 frames => 6 samples
//! let data: Vec<i32> = vec![1, 2, 3, 4, 5, 6];
//!
//! // wrap the data
//! let buffer = InterleavedSlice::new(&data, 2, 3).unwrap();
//!
//! // Loop over all samples and print their values
//! for channel in 0..buffer.channels() {
//!     for frame in 0..buffer.frames() {
//!         let value = buffer.read_sample(channel, frame).unwrap();
//!         println!(
//!             "Channel: {}, frame: {}, value: {}",
//!             channel, frame, value
//!         );
//!     }
//! }
//! ```
//!

use crate::SizeError;
use crate::slicetools::copy_within_slice;
use crate::{check_slice_length, implement_size_getters};
use audioadapter::{Adapter, AdapterMut};

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

macro_rules! check_slice_and_vec_length {
    ($buf:expr, $channels:expr, $frames:expr, sequential) => {
        if $buf.len() < $channels {
            return Err(SizeError::ChannelsContainer {
                actual: $buf.len(),
                required: $channels,
            });
        }
        for (idx, chan) in $buf.iter().enumerate() {
            if chan.len() < $frames {
                return Err(SizeError::ChannelBuffer {
                    index: idx,
                    actual: chan.len(),
                    required: $frames,
                });
            }
        }
    };
    ($buf:expr, $channels:expr, $frames:expr, $mask:expr, sequential) => {
        if $mask.len() != $channels {
            return Err(SizeError::Mask {
                actual: $mask.len(),
                required: $channels,
            });
        }
        if $buf.len() < $channels {
            return Err(SizeError::ChannelsContainer {
                actual: $buf.len(),
                required: $channels,
            });
        }
        for (idx, (chan, active)) in $buf.iter().zip($mask).enumerate() {
            if *active && chan.len() < $frames {
                return Err(SizeError::ChannelBuffer {
                    index: idx,
                    actual: chan.len(),
                    required: $frames,
                });
            }
        }
    };
    ($buf:expr, $channels:expr, $frames:expr, interleaved) => {
        if $buf.len() < $frames {
            return Err(SizeError::FramesContainer {
                actual: $buf.len(),
                required: $frames,
            });
        }
        for (idx, frame) in $buf.iter().enumerate() {
            if frame.len() < $channels {
                return Err(SizeError::FrameBuffer {
                    index: idx,
                    actual: frame.len(),
                    required: $channels,
                });
            }
        }
    };
}
//
// =========================== SequentialSliceOfVecs ===========================
//

/// Wrapper for a slice of length `channels`, containing vectors of length `frames`.
/// Each vector contains the samples for all frames of one channel.
#[cfg(feature = "alloc")]
pub struct SequentialSliceOfVecs<U> {
    buf: U,
    frames: usize,
    channels: usize,
}

#[cfg(feature = "alloc")]
impl<'a, T> SequentialSliceOfVecs<&'a [Vec<T>]> {
    /// Create a new `SequentialSliceOfVecs` to wrap a slice of vectors.
    /// The slice must contain at least `channels` vectors,
    /// and each vector must be at least `frames` long.
    /// They are allowed to be longer than needed,
    /// but these extra frames or channels cannot
    /// be accessed via the trait methods.
    pub fn new(buf: &'a [Vec<T>], channels: usize, frames: usize) -> Result<Self, SizeError> {
        check_slice_and_vec_length!(buf, channels, frames, sequential);
        Ok(Self {
            buf,
            frames,
            channels,
        })
    }
}

#[cfg(feature = "alloc")]
impl<'a, T> SequentialSliceOfVecs<&'a mut [Vec<T>]> {
    /// Create a new `SequentialSliceOfVecs` to wrap a mutable slice of vectors.
    /// The slice must contain at least `channels` vectors,
    /// and each vector must be at least `frames` long.
    /// They are allowed to be longer than needed,
    /// but these extra frames or channels cannot
    /// be accessed via the trait methods.
    pub fn new_mut(
        buf: &'a mut [Vec<T>],
        channels: usize,
        frames: usize,
    ) -> Result<Self, SizeError> {
        check_slice_and_vec_length!(buf, channels, frames, sequential);
        Ok(Self {
            buf,
            frames,
            channels,
        })
    }
}

#[cfg(feature = "alloc")]
unsafe impl<T> Adapter<T> for SequentialSliceOfVecs<&[Vec<T>]>
where
    T: Clone,
{
    unsafe fn read_sample_unchecked(&self, channel: usize, frame: usize) -> T {
        unsafe { self.buf.get_unchecked(channel).get_unchecked(frame).clone() }
    }

    implement_size_getters!();

    fn copy_from_channel_to_slice(&self, channel: usize, skip: usize, slice: &mut [T]) -> usize {
        if channel >= self.channels || skip >= self.frames {
            return 0;
        }
        let frames_to_write = if (self.frames - skip) < slice.len() {
            self.frames - skip
        } else {
            slice.len()
        };
        slice[..frames_to_write].clone_from_slice(&self.buf[channel][skip..skip + frames_to_write]);
        frames_to_write
    }
}

#[cfg(feature = "alloc")]
unsafe impl<T> Adapter<T> for SequentialSliceOfVecs<&mut [Vec<T>]>
where
    T: Clone,
{
    unsafe fn read_sample_unchecked(&self, channel: usize, frame: usize) -> T {
        unsafe { self.buf.get_unchecked(channel).get_unchecked(frame).clone() }
    }

    implement_size_getters!();

    fn copy_from_channel_to_slice(&self, channel: usize, skip: usize, slice: &mut [T]) -> usize {
        if channel >= self.channels || skip >= self.frames {
            return 0;
        }
        let frames_to_write = if (self.frames - skip) < slice.len() {
            self.frames - skip
        } else {
            slice.len()
        };
        slice[..frames_to_write].clone_from_slice(&self.buf[channel][skip..skip + frames_to_write]);
        frames_to_write
    }
}

#[cfg(feature = "alloc")]
unsafe impl<T> AdapterMut<T> for SequentialSliceOfVecs<&mut [Vec<T>]>
where
    T: Clone,
{
    unsafe fn write_sample_unchecked(&mut self, channel: usize, frame: usize, value: &T) -> bool {
        unsafe {
            *self.buf.get_unchecked_mut(channel).get_unchecked_mut(frame) = value.clone();
        }
        false
    }

    fn copy_from_slice_to_channel(
        &mut self,
        channel: usize,
        skip: usize,
        slice: &[T],
    ) -> (usize, usize) {
        if channel >= self.channels || skip >= self.frames {
            return (0, 0);
        }
        let frames_to_read = if (self.frames - skip) < slice.len() {
            self.frames - skip
        } else {
            slice.len()
        };
        self.buf[channel][skip..skip + frames_to_read].clone_from_slice(&slice[..frames_to_read]);
        (frames_to_read, 0)
    }

    fn copy_frames_within(&mut self, src: usize, dest: usize, count: usize) -> Option<usize> {
        if count > self.frames || src > self.frames - count || dest > self.frames - count {
            return None;
        }
        for ch in self.buf.iter_mut() {
            unsafe {
                copy_within_slice(ch, src, dest, count);
            }
        }
        Some(count)
    }

    fn copy_sample_within(
        &mut self,
        source_channel: usize,
        source_frame: usize,
        target_channel: usize,
        target_frame: usize,
    ) -> bool {
        if source_channel >= self.channels
            || source_frame >= self.frames
            || target_channel >= self.channels
            || target_frame >= self.frames
        {
            return false;
        }
        self.buf[target_channel][target_frame] = self.buf[source_channel][source_frame].clone();
        true
    }

    fn swap_samples(
        &mut self,
        channel_a: usize,
        frame_a: usize,
        channel_b: usize,
        frame_b: usize,
    ) -> bool {
        if channel_a >= self.channels
            || frame_a >= self.frames
            || channel_b >= self.channels
            || frame_b >= self.frames
        {
            return false;
        }
        let temp = self.buf[channel_a][frame_a].clone();
        self.buf[channel_a][frame_a] = self.buf[channel_b][frame_b].clone();
        self.buf[channel_b][frame_b] = temp;
        true
    }
}

//
// =========================== SparseSequentialSliceOfVecs ===========================
//

/// Wrapper for a slice of length `channels`, containing vectors of length `frames`.
/// Each vector contains the samples for all frames of one channel.
/// This is similar to [SequentialSliceOfVecs],
/// but here vectors for unused channels may be empty.
/// Reading from an unused channel returns `T::default()`,
/// while writing does nothing.
#[cfg(feature = "alloc")]
pub struct SparseSequentialSliceOfVecs<'a, U: 'a> {
    buf: U,
    frames: usize,
    channels: usize,
    mask: &'a [bool],
}

#[cfg(feature = "alloc")]
impl<'a, T> SparseSequentialSliceOfVecs<'a, &'a [Vec<T>]> {
    /// Create a new `SparseSequentialSliceOfVecs` to wrap a slice of vectors.
    /// The slice must contain at least `channels` vectors.
    /// The vectors for channels that are marked as active
    /// must be at least `frames` long.
    /// They are allowed to be longer than needed,
    /// but these extra frames or channels cannot
    /// be accessed via the trait methods.
    /// Vectors for unused channels are never accessed and can have any length.
    pub fn new(
        buf: &'a [Vec<T>],
        channels: usize,
        frames: usize,
        active_channels_mask: &'a [bool],
    ) -> Result<Self, SizeError> {
        check_slice_and_vec_length!(buf, channels, frames, active_channels_mask, sequential);
        Ok(Self {
            buf,
            frames,
            channels,
            mask: active_channels_mask,
        })
    }
}

#[cfg(feature = "alloc")]
impl<'a, T> SparseSequentialSliceOfVecs<'a, &'a mut [Vec<T>]> {
    /// Create a new `SparseSequentialSliceOfVecs` to wrap a mutable slice of vectors.
    /// The slice must contain at least `channels` vectors.
    /// The vectors for channels that are marked as active
    /// must be at least `frames` long.
    /// They are allowed to be longer than needed,
    /// but these extra frames or channels cannot
    /// be accessed via the trait methods.
    /// Vectors for unused channels are never accessed and can have any length.
    pub fn new_mut(
        buf: &'a mut [Vec<T>],
        channels: usize,
        frames: usize,
        active_channels_mask: &'a [bool],
    ) -> Result<Self, SizeError> {
        check_slice_and_vec_length!(buf, channels, frames, active_channels_mask, sequential);
        Ok(Self {
            buf,
            frames,
            channels,
            mask: active_channels_mask,
        })
    }
}

#[cfg(feature = "alloc")]
unsafe impl<'a, T> Adapter<T> for SparseSequentialSliceOfVecs<'a, &'a [Vec<T>]>
where
    T: Clone + Default,
{
    unsafe fn read_sample_unchecked(&self, channel: usize, frame: usize) -> T {
        if self.mask[channel] {
            return unsafe { self.buf.get_unchecked(channel).get_unchecked(frame).clone() };
        }
        T::default()
    }

    implement_size_getters!();

    fn copy_from_channel_to_slice(&self, channel: usize, skip: usize, slice: &mut [T]) -> usize {
        if channel >= self.channels || !self.mask[channel] || skip >= self.frames {
            return 0;
        }
        let frames_to_write = if (self.frames - skip) < slice.len() {
            self.frames - skip
        } else {
            slice.len()
        };
        slice[..frames_to_write].clone_from_slice(&self.buf[channel][skip..skip + frames_to_write]);
        frames_to_write
    }
}

#[cfg(feature = "alloc")]
unsafe impl<'a, T> Adapter<T> for SparseSequentialSliceOfVecs<'a, &'a mut [Vec<T>]>
where
    T: Clone + Default,
{
    unsafe fn read_sample_unchecked(&self, channel: usize, frame: usize) -> T {
        if self.mask[channel] {
            return unsafe { self.buf.get_unchecked(channel).get_unchecked(frame).clone() };
        }
        T::default()
    }

    implement_size_getters!();

    fn copy_from_channel_to_slice(&self, channel: usize, skip: usize, slice: &mut [T]) -> usize {
        if channel >= self.channels || !self.mask[channel] || skip >= self.frames {
            return 0;
        }
        let frames_to_write = if (self.frames - skip) < slice.len() {
            self.frames - skip
        } else {
            slice.len()
        };
        slice[..frames_to_write].clone_from_slice(&self.buf[channel][skip..skip + frames_to_write]);
        frames_to_write
    }
}

#[cfg(feature = "alloc")]
unsafe impl<'a, T> AdapterMut<T> for SparseSequentialSliceOfVecs<'a, &'a mut [Vec<T>]>
where
    T: Clone + Default,
{
    unsafe fn write_sample_unchecked(&mut self, channel: usize, frame: usize, value: &T) -> bool {
        if self.mask[channel] {
            unsafe {
                *self.buf.get_unchecked_mut(channel).get_unchecked_mut(frame) = value.clone();
            }
        }
        false
    }

    fn copy_from_slice_to_channel(
        &mut self,
        channel: usize,
        skip: usize,
        slice: &[T],
    ) -> (usize, usize) {
        if channel >= self.channels || !self.mask[channel] || skip >= self.frames {
            return (0, 0);
        }
        let frames_to_read = if (self.frames - skip) < slice.len() {
            self.frames - skip
        } else {
            slice.len()
        };
        self.buf[channel][skip..skip + frames_to_read].clone_from_slice(&slice[..frames_to_read]);
        (frames_to_read, 0)
    }

    fn copy_frames_within(&mut self, src: usize, dest: usize, count: usize) -> Option<usize> {
        if count > self.frames || src > self.frames - count || dest > self.frames - count {
            return None;
        }
        for (ch, active) in self.buf.iter_mut().zip(self.mask.iter()) {
            if *active {
                unsafe {
                    copy_within_slice(ch, src, dest, count);
                }
            }
        }
        Some(count)
    }

    fn copy_sample_within(
        &mut self,
        source_channel: usize,
        source_frame: usize,
        target_channel: usize,
        target_frame: usize,
    ) -> bool {
        if source_channel >= self.channels
            || source_frame >= self.frames
            || target_channel >= self.channels
            || target_frame >= self.frames
            || !self.mask[source_channel]
            || !self.mask[target_channel]
        {
            return false;
        }
        self.buf[target_channel][target_frame] = self.buf[source_channel][source_frame].clone();
        true
    }

    fn swap_samples(
        &mut self,
        channel_a: usize,
        frame_a: usize,
        channel_b: usize,
        frame_b: usize,
    ) -> bool {
        if channel_a >= self.channels
            || frame_a >= self.frames
            || channel_b >= self.channels
            || frame_b >= self.frames
            || !self.mask[channel_a]
            || !self.mask[channel_b]
        {
            return false;
        }
        let temp = self.buf[channel_a][frame_a].clone();
        self.buf[channel_a][frame_a] = self.buf[channel_b][frame_b].clone();
        self.buf[channel_b][frame_b] = temp;
        true
    }
}

//
// =========================== SequentialSliceOfSlices ===========================
//

/// Wrapper for a slice of length `channels`, containing slices of length `frames`.
/// Each channel slice contains the samples for all frames of one channel.
pub struct SequentialSliceOfSlices<U> {
    buf: U,
    frames: usize,
    channels: usize,
}

impl<'a, T> SequentialSliceOfSlices<&'a [&'a [T]]> {
    /// Create a new `SequentialSliceOfSlices` to wrap a slice of slices.
    /// The slice must contain at least `channels` slices,
    /// and each channel slice must be at least `frames` long.
    /// They are allowed to be longer than needed,
    /// but these extra frames or channels cannot
    /// be accessed via the trait methods.
    pub fn new(buf: &'a [&'a [T]], channels: usize, frames: usize) -> Result<Self, SizeError> {
        check_slice_and_vec_length!(buf, channels, frames, sequential);
        Ok(Self {
            buf,
            frames,
            channels,
        })
    }
}

impl<'a, T> SequentialSliceOfSlices<&'a mut [&'a mut [T]]> {
    /// Create a new `SequentialSliceOfSlices` to wrap a mutable slice of slices.
    /// The slice must contain at least `channels` slices,
    /// and each channel slice must be at least `frames` long.
    /// They are allowed to be longer than needed,
    /// but these extra frames or channels cannot
    /// be accessed via the trait methods.
    pub fn new_mut(
        buf: &'a mut [&'a mut [T]],
        channels: usize,
        frames: usize,
    ) -> Result<Self, SizeError> {
        check_slice_and_vec_length!(buf, channels, frames, sequential);
        Ok(Self {
            buf,
            frames,
            channels,
        })
    }
}

unsafe impl<'a, T> Adapter<T> for SequentialSliceOfSlices<&'a [&'a [T]]>
where
    T: Clone,
{
    unsafe fn read_sample_unchecked(&self, channel: usize, frame: usize) -> T {
        unsafe { self.buf.get_unchecked(channel).get_unchecked(frame).clone() }
    }

    implement_size_getters!();

    fn copy_from_channel_to_slice(&self, channel: usize, skip: usize, slice: &mut [T]) -> usize {
        if channel >= self.channels || skip >= self.frames {
            return 0;
        }
        let frames_to_write = if (self.frames - skip) < slice.len() {
            self.frames - skip
        } else {
            slice.len()
        };
        slice[..frames_to_write].clone_from_slice(&self.buf[channel][skip..skip + frames_to_write]);
        frames_to_write
    }
}

unsafe impl<'a, T> Adapter<T> for SequentialSliceOfSlices<&'a mut [&'a mut [T]]>
where
    T: Clone,
{
    unsafe fn read_sample_unchecked(&self, channel: usize, frame: usize) -> T {
        unsafe { self.buf.get_unchecked(channel).get_unchecked(frame).clone() }
    }

    implement_size_getters!();

    fn copy_from_channel_to_slice(&self, channel: usize, skip: usize, slice: &mut [T]) -> usize {
        if channel >= self.channels || skip >= self.frames {
            return 0;
        }
        let frames_to_write = if (self.frames - skip) < slice.len() {
            self.frames - skip
        } else {
            slice.len()
        };
        slice[..frames_to_write].clone_from_slice(&self.buf[channel][skip..skip + frames_to_write]);
        frames_to_write
    }
}

unsafe impl<'a, T> AdapterMut<T> for SequentialSliceOfSlices<&'a mut [&'a mut [T]]>
where
    T: Clone,
{
    unsafe fn write_sample_unchecked(&mut self, channel: usize, frame: usize, value: &T) -> bool {
        unsafe {
            *self.buf.get_unchecked_mut(channel).get_unchecked_mut(frame) = value.clone();
        }
        false
    }

    fn copy_from_slice_to_channel(
        &mut self,
        channel: usize,
        skip: usize,
        slice: &[T],
    ) -> (usize, usize) {
        if channel >= self.channels || skip >= self.frames {
            return (0, 0);
        }
        let frames_to_read = if (self.frames - skip) < slice.len() {
            self.frames - skip
        } else {
            slice.len()
        };
        self.buf[channel][skip..skip + frames_to_read].clone_from_slice(&slice[..frames_to_read]);
        (frames_to_read, 0)
    }

    fn copy_frames_within(&mut self, src: usize, dest: usize, count: usize) -> Option<usize> {
        if count > self.frames || src > self.frames - count || dest > self.frames - count {
            return None;
        }
        for ch in self.buf.iter_mut() {
            unsafe {
                copy_within_slice(ch, src, dest, count);
            }
        }
        Some(count)
    }

    fn copy_sample_within(
        &mut self,
        source_channel: usize,
        source_frame: usize,
        target_channel: usize,
        target_frame: usize,
    ) -> bool {
        if source_channel >= self.channels
            || source_frame >= self.frames
            || target_channel >= self.channels
            || target_frame >= self.frames
        {
            return false;
        }
        self.buf[target_channel][target_frame] = self.buf[source_channel][source_frame].clone();
        true
    }

    fn swap_samples(
        &mut self,
        channel_a: usize,
        frame_a: usize,
        channel_b: usize,
        frame_b: usize,
    ) -> bool {
        if channel_a >= self.channels
            || frame_a >= self.frames
            || channel_b >= self.channels
            || frame_b >= self.frames
        {
            return false;
        }
        let temp = self.buf[channel_a][frame_a].clone();
        self.buf[channel_a][frame_a] = self.buf[channel_b][frame_b].clone();
        self.buf[channel_b][frame_b] = temp;
        true
    }
}

//
// =========================== SparseSequentialSliceOfSlices ===========================
//

/// Wrapper for a slice of length `channels`, containing slices of length `frames`.
/// Each channel slice contains the samples for all frames of one channel.
/// This is similar to [SequentialSliceOfSlices],
/// but here slices for unused channels may be empty.
/// Reading from an unused channel returns `T::default()`,
/// while writing does nothing.
pub struct SparseSequentialSliceOfSlices<'a, U: 'a> {
    buf: U,
    frames: usize,
    channels: usize,
    mask: &'a [bool],
}

impl<'a, T> SparseSequentialSliceOfSlices<'a, &'a [&'a [T]]> {
    /// Create a new `SparseSequentialSliceOfSlices` to wrap a slice of slices.
    /// The slice must contain at least `channels` slices.
    /// The slices for channels that are marked as active
    /// must be at least `frames` long.
    /// They are allowed to be longer than needed,
    /// but these extra frames or channels cannot
    /// be accessed via the trait methods.
    /// Slices for unused channels are never accessed and can have any length.
    pub fn new(
        buf: &'a [&'a [T]],
        channels: usize,
        frames: usize,
        active_channels_mask: &'a [bool],
    ) -> Result<Self, SizeError> {
        check_slice_and_vec_length!(buf, channels, frames, active_channels_mask, sequential);
        Ok(Self {
            buf,
            frames,
            channels,
            mask: active_channels_mask,
        })
    }
}

impl<'a, T> SparseSequentialSliceOfSlices<'a, &'a mut [&'a mut [T]]> {
    /// Create a new `SparseSequentialSliceOfSlices` to wrap a mutable slice of slices.
    /// The slice must contain at least `channels` slices.
    /// The slices for channels that are marked as active
    /// must be at least `frames` long.
    /// They are allowed to be longer than needed,
    /// but these extra frames or channels cannot
    /// be accessed via the trait methods.
    /// Slices for unused channels are never accessed and can have any length.
    pub fn new_mut(
        buf: &'a mut [&'a mut [T]],
        channels: usize,
        frames: usize,
        active_channels_mask: &'a [bool],
    ) -> Result<Self, SizeError> {
        check_slice_and_vec_length!(buf, channels, frames, active_channels_mask, sequential);
        Ok(Self {
            buf,
            frames,
            channels,
            mask: active_channels_mask,
        })
    }
}

unsafe impl<'a, T> Adapter<T> for SparseSequentialSliceOfSlices<'a, &'a [&'a [T]]>
where
    T: Clone + Default,
{
    unsafe fn read_sample_unchecked(&self, channel: usize, frame: usize) -> T {
        if self.mask[channel] {
            return unsafe { self.buf.get_unchecked(channel).get_unchecked(frame).clone() };
        }
        T::default()
    }

    implement_size_getters!();

    fn copy_from_channel_to_slice(&self, channel: usize, skip: usize, slice: &mut [T]) -> usize {
        if channel >= self.channels || !self.mask[channel] || skip >= self.frames {
            return 0;
        }
        let frames_to_write = if (self.frames - skip) < slice.len() {
            self.frames - skip
        } else {
            slice.len()
        };
        slice[..frames_to_write].clone_from_slice(&self.buf[channel][skip..skip + frames_to_write]);
        frames_to_write
    }
}

unsafe impl<'a, T> Adapter<T> for SparseSequentialSliceOfSlices<'a, &'a mut [&'a mut [T]]>
where
    T: Clone + Default,
{
    unsafe fn read_sample_unchecked(&self, channel: usize, frame: usize) -> T {
        if self.mask[channel] {
            return unsafe { self.buf.get_unchecked(channel).get_unchecked(frame).clone() };
        }
        T::default()
    }

    implement_size_getters!();

    fn copy_from_channel_to_slice(&self, channel: usize, skip: usize, slice: &mut [T]) -> usize {
        if channel >= self.channels || !self.mask[channel] || skip >= self.frames {
            return 0;
        }
        let frames_to_write = if (self.frames - skip) < slice.len() {
            self.frames - skip
        } else {
            slice.len()
        };
        slice[..frames_to_write].clone_from_slice(&self.buf[channel][skip..skip + frames_to_write]);
        frames_to_write
    }
}

unsafe impl<'a, T> AdapterMut<T> for SparseSequentialSliceOfSlices<'a, &'a mut [&'a mut [T]]>
where
    T: Clone + Default,
{
    unsafe fn write_sample_unchecked(&mut self, channel: usize, frame: usize, value: &T) -> bool {
        if self.mask[channel] {
            unsafe {
                *self.buf.get_unchecked_mut(channel).get_unchecked_mut(frame) = value.clone();
            }
        }
        false
    }

    fn copy_from_slice_to_channel(
        &mut self,
        channel: usize,
        skip: usize,
        slice: &[T],
    ) -> (usize, usize) {
        if channel >= self.channels || !self.mask[channel] || skip >= self.frames {
            return (0, 0);
        }
        let frames_to_read = if (self.frames - skip) < slice.len() {
            self.frames - skip
        } else {
            slice.len()
        };
        self.buf[channel][skip..skip + frames_to_read].clone_from_slice(&slice[..frames_to_read]);
        (frames_to_read, 0)
    }

    fn copy_frames_within(&mut self, src: usize, dest: usize, count: usize) -> Option<usize> {
        if count > self.frames || src > self.frames - count || dest > self.frames - count {
            return None;
        }
        for (ch, active) in self.buf.iter_mut().zip(self.mask.iter()) {
            if *active {
                unsafe {
                    copy_within_slice(ch, src, dest, count);
                }
            }
        }
        Some(count)
    }

    fn copy_sample_within(
        &mut self,
        source_channel: usize,
        source_frame: usize,
        target_channel: usize,
        target_frame: usize,
    ) -> bool {
        if source_channel >= self.channels
            || source_frame >= self.frames
            || target_channel >= self.channels
            || target_frame >= self.frames
            || !self.mask[source_channel]
            || !self.mask[target_channel]
        {
            return false;
        }
        self.buf[target_channel][target_frame] = self.buf[source_channel][source_frame].clone();
        true
    }

    fn swap_samples(
        &mut self,
        channel_a: usize,
        frame_a: usize,
        channel_b: usize,
        frame_b: usize,
    ) -> bool {
        if channel_a >= self.channels
            || frame_a >= self.frames
            || channel_b >= self.channels
            || frame_b >= self.frames
            || !self.mask[channel_a]
            || !self.mask[channel_b]
        {
            return false;
        }
        let temp = self.buf[channel_a][frame_a].clone();
        self.buf[channel_a][frame_a] = self.buf[channel_b][frame_b].clone();
        self.buf[channel_b][frame_b] = temp;
        true
    }
}

//
// =========================== InterleavedSliceOfVecs ===========================
//

/// Wrapper for a slice of length `frames`, containing vectors of length `channels`.
/// Each vector contains the samples for all channels of one frame.
#[cfg(feature = "alloc")]
pub struct InterleavedSliceOfVecs<U> {
    buf: U,
    frames: usize,
    channels: usize,
}

#[cfg(feature = "alloc")]
impl<'a, T> InterleavedSliceOfVecs<&'a [Vec<T>]> {
    /// Create a new `InterleavedSliceOfVecs` to wrap a slice of vectors.
    /// The slice must contain at least `frames` vectors,
    /// and each vector must be at least `channels` long.
    /// They are allowed to be longer than needed,
    /// but these extra frames or channels cannot
    /// be accessed via the trait methods.
    pub fn new(buf: &'a [Vec<T>], channels: usize, frames: usize) -> Result<Self, SizeError> {
        check_slice_and_vec_length!(buf, channels, frames, interleaved);
        Ok(Self {
            buf,
            frames,
            channels,
        })
    }
}

#[cfg(feature = "alloc")]
impl<'a, T> InterleavedSliceOfVecs<&'a mut [Vec<T>]> {
    /// Create a new `InterleavedSliceOfVecs` to wrap a mutable slice of vectors.
    /// The slice must contain at least `frames` vectors,
    /// and each vector must be at least `channels` long.
    /// They are allowed to be longer than needed,
    /// but these extra frames or channels cannot
    /// be accessed via the trait methods.
    pub fn new_mut(
        buf: &'a mut [Vec<T>],
        channels: usize,
        frames: usize,
    ) -> Result<Self, SizeError> {
        check_slice_and_vec_length!(buf, channels, frames, interleaved);
        Ok(Self {
            buf,
            frames,
            channels,
        })
    }
}

#[cfg(feature = "alloc")]
unsafe impl<T> Adapter<T> for InterleavedSliceOfVecs<&[Vec<T>]>
where
    T: Clone,
{
    unsafe fn read_sample_unchecked(&self, channel: usize, frame: usize) -> T {
        unsafe { self.buf.get_unchecked(frame).get_unchecked(channel).clone() }
    }

    implement_size_getters!();

    fn copy_from_frame_to_slice(&self, frame: usize, skip: usize, slice: &mut [T]) -> usize {
        if frame >= self.frames || skip >= self.channels {
            return 0;
        }
        let channels_to_write = if (self.channels - skip) < slice.len() {
            self.channels - skip
        } else {
            slice.len()
        };
        slice[..channels_to_write]
            .clone_from_slice(&self.buf[frame][skip..skip + channels_to_write]);
        channels_to_write
    }
}

#[cfg(feature = "alloc")]
unsafe impl<T> Adapter<T> for InterleavedSliceOfVecs<&mut [Vec<T>]>
where
    T: Clone,
{
    unsafe fn read_sample_unchecked(&self, channel: usize, frame: usize) -> T {
        unsafe { self.buf.get_unchecked(frame).get_unchecked(channel).clone() }
    }

    implement_size_getters!();

    fn copy_from_frame_to_slice(&self, frame: usize, skip: usize, slice: &mut [T]) -> usize {
        if frame >= self.frames || skip >= self.channels {
            return 0;
        }
        let channels_to_write = if (self.channels - skip) < slice.len() {
            self.channels - skip
        } else {
            slice.len()
        };
        slice[..channels_to_write]
            .clone_from_slice(&self.buf[frame][skip..skip + channels_to_write]);
        channels_to_write
    }
}

#[cfg(feature = "alloc")]
unsafe impl<T> AdapterMut<T> for InterleavedSliceOfVecs<&mut [Vec<T>]>
where
    T: Clone,
{
    unsafe fn write_sample_unchecked(&mut self, channel: usize, frame: usize, value: &T) -> bool {
        unsafe {
            *self.buf.get_unchecked_mut(frame).get_unchecked_mut(channel) = value.clone();
        }
        false
    }

    fn copy_from_slice_to_frame(
        &mut self,
        frame: usize,
        skip: usize,
        slice: &[T],
    ) -> (usize, usize) {
        if frame >= self.frames || skip >= self.channels {
            return (0, 0);
        }
        let channels_to_read = if (self.channels - skip) < slice.len() {
            self.channels - skip
        } else {
            slice.len()
        };
        self.buf[frame][skip..skip + channels_to_read].clone_from_slice(&slice[..channels_to_read]);
        (channels_to_read, 0)
    }

    fn copy_sample_within(
        &mut self,
        source_channel: usize,
        source_frame: usize,
        target_channel: usize,
        target_frame: usize,
    ) -> bool {
        if source_channel >= self.channels
            || source_frame >= self.frames
            || target_channel >= self.channels
            || target_frame >= self.frames
        {
            return false;
        }
        self.buf[target_frame][target_channel] = self.buf[source_frame][source_channel].clone();
        true
    }

    fn swap_samples(
        &mut self,
        channel_a: usize,
        frame_a: usize,
        channel_b: usize,
        frame_b: usize,
    ) -> bool {
        if channel_a >= self.channels
            || frame_a >= self.frames
            || channel_b >= self.channels
            || frame_b >= self.frames
        {
            return false;
        }
        let temp = self.buf[frame_a][channel_a].clone();
        self.buf[frame_a][channel_a] = self.buf[frame_b][channel_b].clone();
        self.buf[frame_b][channel_b] = temp;
        true
    }
}

//
// =========================== InterleavedSlice ===========================
//

/// Wrapper for a slice of length `frames * channels`.
/// The samples are stored in _interleaved_ order,
/// where all the samples for one frame are stored consecutively,
/// followed by the samples for the next frame.
/// For a stereo buffer containing four frames, the order is
/// `L1, R1, L2, R2, L3, R3, L4, R4`
pub struct InterleavedSlice<U> {
    buf: U,
    frames: usize,
    channels: usize,
}

impl<U> InterleavedSlice<U> {
    fn calc_index(&self, channel: usize, frame: usize) -> usize {
        frame * self.channels + channel
    }
}

impl<'a, T> InterleavedSlice<&'a [T]> {
    /// Create a new `InterleavedSlice` to wrap a slice.
    /// The slice length must be at least `frames*channels`.
    /// It is allowed to be longer than needed,
    /// but these extra values cannot
    /// be accessed via the trait methods.
    pub fn new(buf: &'a [T], channels: usize, frames: usize) -> Result<Self, SizeError> {
        check_slice_length!(channels, frames, buf.len());
        Ok(Self {
            buf,
            frames,
            channels,
        })
    }
}

impl<'a, T> InterleavedSlice<&'a mut [T]> {
    /// Create a new `InterleavedSlice` to wrap a mutable slice.
    /// The slice length must be at least `frames*channels`.
    /// It is allowed to be longer than needed,
    /// but these extra values cannot
    /// be accessed via the trait methods.
    pub fn new_mut(buf: &'a mut [T], channels: usize, frames: usize) -> Result<Self, SizeError> {
        check_slice_length!(channels, frames, buf.len());
        Ok(Self {
            buf,
            frames,
            channels,
        })
    }
}

unsafe impl<T> Adapter<T> for InterleavedSlice<&[T]>
where
    T: Clone,
{
    unsafe fn read_sample_unchecked(&self, channel: usize, frame: usize) -> T {
        let index = self.calc_index(channel, frame);
        unsafe { self.buf.get_unchecked(index).clone() }
    }

    implement_size_getters!();

    fn copy_from_frame_to_slice(&self, frame: usize, skip: usize, slice: &mut [T]) -> usize {
        if frame >= self.frames || skip >= self.channels {
            return 0;
        }
        let channels_to_write = if (self.channels - skip) < slice.len() {
            self.channels - skip
        } else {
            slice.len()
        };
        let buffer_skip = self.calc_index(skip, frame);
        slice[..channels_to_write]
            .clone_from_slice(&self.buf[buffer_skip..buffer_skip + channels_to_write]);
        channels_to_write
    }
}

unsafe impl<T> Adapter<T> for InterleavedSlice<&mut [T]>
where
    T: Clone,
{
    unsafe fn read_sample_unchecked(&self, channel: usize, frame: usize) -> T {
        let index = self.calc_index(channel, frame);
        unsafe { self.buf.get_unchecked(index).clone() }
    }

    implement_size_getters!();

    fn copy_from_frame_to_slice(&self, frame: usize, skip: usize, slice: &mut [T]) -> usize {
        if frame >= self.frames || skip >= self.channels {
            return 0;
        }
        let channels_to_write = if (self.channels - skip) < slice.len() {
            self.channels - skip
        } else {
            slice.len()
        };
        let buffer_skip = self.calc_index(skip, frame);
        slice[..channels_to_write]
            .clone_from_slice(&self.buf[buffer_skip..buffer_skip + channels_to_write]);
        channels_to_write
    }
}

unsafe impl<T> AdapterMut<T> for InterleavedSlice<&mut [T]>
where
    T: Clone,
{
    unsafe fn write_sample_unchecked(&mut self, channel: usize, frame: usize, value: &T) -> bool {
        let index = self.calc_index(channel, frame);
        unsafe {
            *self.buf.get_unchecked_mut(index) = value.clone();
        }
        false
    }

    fn copy_from_slice_to_frame(
        &mut self,
        frame: usize,
        skip: usize,
        slice: &[T],
    ) -> (usize, usize) {
        if frame >= self.frames || skip >= self.channels {
            return (0, 0);
        }
        let channels_to_read = if (self.channels - skip) < slice.len() {
            self.channels - skip
        } else {
            slice.len()
        };
        let buffer_skip = self.calc_index(skip, frame);
        self.buf[buffer_skip..buffer_skip + channels_to_read]
            .clone_from_slice(&slice[..channels_to_read]);
        (channels_to_read, 0)
    }

    fn copy_frames_within(&mut self, src: usize, dest: usize, count: usize) -> Option<usize> {
        if count > self.frames || src > self.frames - count || dest > self.frames - count {
            return None;
        }
        unsafe {
            copy_within_slice(
                self.buf,
                src * self.channels,
                dest * self.channels,
                count * self.channels,
            );
        }
        Some(count)
    }

    fn copy_sample_within(
        &mut self,
        source_channel: usize,
        source_frame: usize,
        target_channel: usize,
        target_frame: usize,
    ) -> bool {
        if source_channel >= self.channels
            || source_frame >= self.frames
            || target_channel >= self.channels
            || target_frame >= self.frames
        {
            return false;
        }
        let source_index = self.calc_index(source_channel, source_frame);
        let target_index = self.calc_index(target_channel, target_frame);
        self.buf[target_index] = self.buf[source_index].clone();
        true
    }

    fn swap_samples(
        &mut self,
        channel_a: usize,
        frame_a: usize,
        channel_b: usize,
        frame_b: usize,
    ) -> bool {
        if channel_a >= self.channels
            || frame_a >= self.frames
            || channel_b >= self.channels
            || frame_b >= self.frames
        {
            return false;
        }
        let index_a = self.calc_index(channel_a, frame_a);
        let index_b = self.calc_index(channel_b, frame_b);
        self.buf.swap(index_a, index_b);
        true
    }
}

//
// =========================== SequentialSlice ===========================
//

/// Wrapper for a slice of length `frames * channels`.
/// The samples are stored in _sequential_ order,
/// where all the samples for one channel are stored consecutively,
/// followed by the samples for the next channel.
/// For a stereo buffer containing four frames, the order is
/// `L1, L2, L3, L4, R1, R2, R3, R4`
pub struct SequentialSlice<U> {
    buf: U,
    frames: usize,
    channels: usize,
}

impl<U> SequentialSlice<U> {
    fn calc_index(&self, channel: usize, frame: usize) -> usize {
        channel * self.frames + frame
    }
}

impl<'a, T> SequentialSlice<&'a [T]> {
    /// Create a new `SequentialSlice` to wrap a slice.
    /// The slice length must be at least `frames*channels`.
    /// It is allowed to be longer than needed,
    /// but these extra values cannot
    /// be accessed via the trait methods.
    pub fn new(buf: &'a [T], channels: usize, frames: usize) -> Result<Self, SizeError> {
        check_slice_length!(channels, frames, buf.len());
        Ok(Self {
            buf,
            frames,
            channels,
        })
    }
}

impl<'a, T> SequentialSlice<&'a mut [T]> {
    /// Create a new `SequentialSlice` to wrap a mutable slice.
    /// The slice length must be at least `frames*channels`.
    /// It is allowed to be longer than needed,
    /// but these extra values cannot
    /// be accessed via the trait methods.
    pub fn new_mut(buf: &'a mut [T], channels: usize, frames: usize) -> Result<Self, SizeError> {
        check_slice_length!(channels, frames, buf.len());
        Ok(Self {
            buf,
            frames,
            channels,
        })
    }
}

unsafe impl<T> Adapter<T> for SequentialSlice<&[T]>
where
    T: Clone,
{
    unsafe fn read_sample_unchecked(&self, channel: usize, frame: usize) -> T {
        let index = self.calc_index(channel, frame);
        unsafe { self.buf.get_unchecked(index).clone() }
    }

    implement_size_getters!();

    fn copy_from_channel_to_slice(&self, channel: usize, skip: usize, slice: &mut [T]) -> usize {
        if channel >= self.channels || skip >= self.frames {
            return 0;
        }
        let frames_to_write = if (self.frames - skip) < slice.len() {
            self.frames - skip
        } else {
            slice.len()
        };
        let buffer_skip = self.calc_index(channel, skip);
        slice[..frames_to_write]
            .clone_from_slice(&self.buf[buffer_skip..buffer_skip + frames_to_write]);
        frames_to_write
    }
}

// Implement also for mutable version, identical to the immutable impl.
unsafe impl<T> Adapter<T> for SequentialSlice<&mut [T]>
where
    T: Clone,
{
    unsafe fn read_sample_unchecked(&self, channel: usize, frame: usize) -> T {
        let index = self.calc_index(channel, frame);
        unsafe { self.buf.get_unchecked(index).clone() }
    }

    implement_size_getters!();

    fn copy_from_channel_to_slice(&self, channel: usize, skip: usize, slice: &mut [T]) -> usize {
        if channel >= self.channels || skip >= self.frames {
            return 0;
        }
        let frames_to_write = if (self.frames - skip) < slice.len() {
            self.frames - skip
        } else {
            slice.len()
        };
        let buffer_skip = self.calc_index(channel, skip);
        slice[..frames_to_write]
            .clone_from_slice(&self.buf[buffer_skip..buffer_skip + frames_to_write]);
        frames_to_write
    }
}

unsafe impl<T> AdapterMut<T> for SequentialSlice<&mut [T]>
where
    T: Clone,
{
    unsafe fn write_sample_unchecked(&mut self, channel: usize, frame: usize, value: &T) -> bool {
        let index = self.calc_index(channel, frame);
        unsafe {
            *self.buf.get_unchecked_mut(index) = value.clone();
        }
        false
    }

    fn copy_from_slice_to_channel(
        &mut self,
        channel: usize,
        skip: usize,
        slice: &[T],
    ) -> (usize, usize) {
        if channel >= self.channels || skip >= self.frames {
            return (0, 0);
        }
        let frames_to_read = if (self.frames - skip) < slice.len() {
            self.frames - skip
        } else {
            slice.len()
        };
        let buffer_skip = self.calc_index(channel, skip);
        self.buf[buffer_skip..buffer_skip + frames_to_read]
            .clone_from_slice(&slice[..frames_to_read]);
        (frames_to_read, 0)
    }

    fn copy_frames_within(&mut self, src: usize, dest: usize, count: usize) -> Option<usize> {
        if count > self.frames || src > self.frames - count || dest > self.frames - count {
            return None;
        }
        for ch in 0..self.channels {
            let offset = ch * self.frames;
            unsafe {
                copy_within_slice(self.buf, src + offset, dest + offset, count);
            }
        }
        Some(count)
    }

    fn copy_sample_within(
        &mut self,
        source_channel: usize,
        source_frame: usize,
        target_channel: usize,
        target_frame: usize,
    ) -> bool {
        if source_channel >= self.channels
            || source_frame >= self.frames
            || target_channel >= self.channels
            || target_frame >= self.frames
        {
            return false;
        }
        let source_index = self.calc_index(source_channel, source_frame);
        let target_index = self.calc_index(target_channel, target_frame);
        self.buf[target_index] = self.buf[source_index].clone();
        true
    }

    fn swap_samples(
        &mut self,
        channel_a: usize,
        frame_a: usize,
        channel_b: usize,
        frame_b: usize,
    ) -> bool {
        if channel_a >= self.channels
            || frame_a >= self.frames
            || channel_b >= self.channels
            || frame_b >= self.frames
        {
            return false;
        }
        let index_a = self.calc_index(channel_a, frame_a);
        let index_b = self.calc_index(channel_b, frame_b);
        self.buf.swap(index_a, index_b);
        true
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
    use audioadapter::tests::test_adapter_mut_methods;
    extern crate alloc;
    use alloc::vec;

    #[cfg(feature = "alloc")]
    use alloc::boxed::Box;

    fn insert_data(buffer: &mut dyn AdapterMut<i32>) {
        buffer.write_sample(0, 0, &1).unwrap();
        buffer.write_sample(0, 1, &2).unwrap();
        buffer.write_sample(0, 2, &3).unwrap();
        buffer.write_sample(1, 0, &4).unwrap();
        buffer.write_sample(1, 1, &5).unwrap();
        buffer.write_sample(1, 2, &6).unwrap();
    }

    fn test_get(buffer: &mut dyn AdapterMut<i32>) {
        insert_data(buffer);
        assert_eq!(buffer.read_sample(0, 0).unwrap(), 1);
        assert_eq!(buffer.read_sample(0, 1).unwrap(), 2);
        assert_eq!(buffer.read_sample(0, 2).unwrap(), 3);
        assert_eq!(buffer.read_sample(1, 0).unwrap(), 4);
        assert_eq!(buffer.read_sample(1, 1).unwrap(), 5);
        assert_eq!(buffer.read_sample(1, 2).unwrap(), 6);
    }

    fn test_slice_channel(buffer: &mut dyn AdapterMut<i32>) {
        insert_data(buffer);
        let mut other1 = [0; 2];
        let mut other2 = [0; 4];
        buffer.copy_from_channel_to_slice(0, 1, &mut other1);
        buffer.copy_from_channel_to_slice(1, 0, &mut other2);
        assert_eq!(other1[0], 2);
        assert_eq!(other1[1], 3);
        assert_eq!(other2[0], 4);
        assert_eq!(other2[1], 5);
        assert_eq!(other2[2], 6);
        assert_eq!(other2[3], 0);
    }

    fn test_slice_frame(buffer: &mut dyn AdapterMut<i32>) {
        insert_data(buffer);
        let mut other1 = [0; 1];
        let mut other2 = [0; 3];
        buffer.copy_from_frame_to_slice(0, 1, &mut other1);
        buffer.copy_from_frame_to_slice(1, 0, &mut other2);
        assert_eq!(other1[0], 4);
        assert_eq!(other2[0], 2);
        assert_eq!(other2[1], 5);
        assert_eq!(other2[2], 0);
    }

    fn test_mut_slice_channel(buffer: &mut dyn AdapterMut<i32>) {
        insert_data(buffer);
        let other1 = [8, 9];
        let other2 = [10, 11, 12, 13];
        buffer.copy_from_slice_to_channel(0, 1, &other1);
        buffer.copy_from_slice_to_channel(1, 0, &other2);
        assert_eq!(buffer.read_sample(0, 0).unwrap(), 1);
        assert_eq!(buffer.read_sample(0, 1).unwrap(), 8);
        assert_eq!(buffer.read_sample(0, 2).unwrap(), 9);
        assert_eq!(buffer.read_sample(1, 0).unwrap(), 10);
        assert_eq!(buffer.read_sample(1, 1).unwrap(), 11);
        assert_eq!(buffer.read_sample(1, 2).unwrap(), 12);
    }

    fn test_mut_slice_frame(buffer: &mut dyn AdapterMut<i32>) {
        insert_data(buffer);
        let other1 = [8];
        let other2 = [10, 11, 12];
        buffer.copy_from_slice_to_frame(0, 0, &other1);
        buffer.copy_from_slice_to_frame(1, 0, &other2);
        assert_eq!(buffer.read_sample(0, 0).unwrap(), 8);
        assert_eq!(buffer.read_sample(1, 0).unwrap(), 4);
        assert_eq!(buffer.read_sample(0, 1).unwrap(), 10);
        assert_eq!(buffer.read_sample(1, 1).unwrap(), 11);
        assert_eq!(buffer.read_sample(0, 2).unwrap(), 3);
        assert_eq!(buffer.read_sample(1, 2).unwrap(), 6);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn vec_of_vec_channels() {
        let mut data = vec![vec![0_i32; 3], vec![0_i32; 3]];
        let mut buffer = SequentialSliceOfVecs::new_mut(&mut data, 2, 3).unwrap();
        test_get(&mut buffer);
        test_slice_channel(&mut buffer);
        test_slice_frame(&mut buffer);
        test_mut_slice_channel(&mut buffer);
        test_mut_slice_frame(&mut buffer);
    }

    #[test]
    fn slice_of_slice_channels() {
        let mut data_ch_0 = vec![0_i32; 3];
        let mut data_ch_1 = vec![0_i32; 3];
        let mut data = vec![data_ch_0.as_mut_slice(), data_ch_1.as_mut_slice()];

        let mut buffer = SequentialSliceOfSlices::new_mut(&mut data, 2, 3).unwrap();
        test_get(&mut buffer);
        test_slice_channel(&mut buffer);
        test_slice_frame(&mut buffer);
        test_mut_slice_channel(&mut buffer);
        test_mut_slice_frame(&mut buffer);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn vec_of_frames() {
        let mut data = vec![vec![1_i32, 4], vec![2_i32, 5], vec![3, 6]];
        let mut buffer = InterleavedSliceOfVecs::new_mut(&mut data, 2, 3).unwrap();
        test_get(&mut buffer);
        test_slice_channel(&mut buffer);
        test_slice_frame(&mut buffer);
        test_mut_slice_channel(&mut buffer);
        test_mut_slice_frame(&mut buffer);
    }

    #[test]
    fn interleaved() {
        let mut data = [1_i32, 4, 2, 5, 3, 6];
        let mut buffer = InterleavedSlice::new_mut(&mut data, 2, 3).unwrap();
        test_get(&mut buffer);
        test_slice_channel(&mut buffer);
        test_slice_frame(&mut buffer);
        test_mut_slice_channel(&mut buffer);
        test_mut_slice_frame(&mut buffer);
    }

    #[test]
    fn sequential() {
        let mut data = [1_i32, 2, 3, 4, 5, 6];
        let mut buffer = SequentialSlice::new_mut(&mut data, 2, 3).unwrap();
        test_get(&mut buffer);
        test_slice_channel(&mut buffer);
        test_slice_frame(&mut buffer);
        test_mut_slice_channel(&mut buffer);
        test_mut_slice_frame(&mut buffer);
    }

    // This tests that an Adapter is object safe.
    #[cfg(feature = "alloc")]
    #[test]
    fn boxed_buffer() {
        let data = [1_i32, 2, 3, 4, 5, 6];
        let boxed: Box<dyn Adapter<i32>> = Box::new(SequentialSlice::new(&data, 2, 3).unwrap());
        assert_eq!(boxed.read_sample(0, 0).unwrap(), 1);
    }

    // Check that a buffer is Send + Sync,
    // meaning it can be sent between threads.
    // This test is not designed to be run, only to compile.
    #[allow(dead_code)]
    fn test_adapter_send_and_sync() {
        fn is_send<T: Send>() {}
        fn is_sync<T: Sync>() {}
        is_send::<InterleavedSlice<f32>>();
        is_sync::<InterleavedSlice<f32>>();
        #[cfg(feature = "alloc")]
        is_send::<InterleavedSliceOfVecs<f32>>();
        #[cfg(feature = "alloc")]
        is_sync::<InterleavedSliceOfVecs<f32>>();
    }

    #[test]
    fn copy_channel_from_other() {
        let data_other = [1.0_f32, 2.0, 3.0, 4.0, 5.0, 6.0];
        let other = SequentialSlice::new(&data_other, 2, 3).unwrap();
        let mut data = [0.0; 6];
        let mut buffer = SequentialSlice::new_mut(&mut data, 2, 3).unwrap();
        // copy second and third element of second channel of other to first and second element of first channel
        let res1 = buffer.copy_from_other_to_channel(&other, 1, 0, 1, 0, 2);
        // copy first and second element of first channel of other to second and third element of second channel
        let res2 = buffer.copy_from_other_to_channel(&other, 0, 1, 0, 1, 2);
        assert_eq!(res1, Some(0));
        assert_eq!(res2, Some(0));
        assert_eq!(buffer.read_sample(0, 0).unwrap(), 5.0);
        assert_eq!(buffer.read_sample(0, 1).unwrap(), 6.0);
        assert_eq!(buffer.read_sample(0, 2).unwrap(), 0.0);
        assert_eq!(buffer.read_sample(1, 0).unwrap(), 0.0);
        assert_eq!(buffer.read_sample(1, 1).unwrap(), 1.0);
        assert_eq!(buffer.read_sample(1, 2).unwrap(), 2.0);
    }

    #[test]
    fn fill_channel() {
        let mut data: [i32; 6] = [1; 6];
        let mut buffer = InterleavedSlice::new_mut(&mut data, 2, 3).unwrap();
        buffer.fill_channel_with(0, &2);
        let expected: [i32; 6] = [2, 1, 2, 1, 2, 1];
        assert_eq!(data, expected);
    }

    #[test]
    fn fill_frame() {
        let mut data: [i32; 6] = [1; 6];
        let mut buffer = InterleavedSlice::new_mut(&mut data, 2, 3).unwrap();
        buffer.fill_frame_with(1, &2);
        let expected: [i32; 6] = [1, 1, 2, 2, 1, 1];
        assert_eq!(data, expected);
    }

    #[test]
    fn fill_buffer() {
        let mut data: [i32; 6] = [1; 6];
        let mut buffer = InterleavedSlice::new_mut(&mut data, 2, 3).unwrap();
        buffer.fill_with(&2);
        let expected: [i32; 6] = [2; 6];
        assert_eq!(data, expected);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn sparse_sequential_vecs() {
        use audioadapter::stats::AdapterStats;

        let mut data = vec![vec![1, 2, 3], Vec::new()];
        let mask = vec![true, false];
        let mut buffer = SparseSequentialSliceOfVecs::new_mut(&mut data, 2, 3, &mask).unwrap();
        // Read active channel gives the proper value
        assert_eq!(buffer.read_sample(0, 1), Some(2));
        // Reading unused channel gives zero
        assert_eq!(buffer.read_sample(1, 1), Some(0));
        // write and read an active channel
        assert_eq!(buffer.write_sample(0, 1, &25), Some(false));
        assert_eq!(buffer.read_sample(0, 1), Some(25));
        // write to an unused channel is successful (but does nothing)
        assert_eq!(buffer.write_sample(1, 1, &26), Some(false));
        // reading outside the actual size gives None
        assert_eq!(buffer.read_sample(0, 10), None);
        assert_eq!(buffer.read_sample(1, 10), None);
        assert_eq!(buffer.read_sample(2, 1), None);
        // RMS of the active channel should be 14.55
        assert!((buffer.channel_rms(0) - 14.5).abs() < 0.1);
        // RMS of the unused channel should be zero
        assert_eq!(buffer.channel_rms(1), 0.0);
    }

    #[test]
    fn sparse_sequential_slices() {
        use audioadapter::stats::AdapterStats;

        let mut data_ch_0 = vec![1, 2, 3];

        let mut data = vec![data_ch_0.as_mut_slice(), &mut []];
        let mask = vec![true, false];
        let mut buffer = SparseSequentialSliceOfSlices::new_mut(&mut data, 2, 3, &mask).unwrap();
        // Read active channel gives the proper value
        assert_eq!(buffer.read_sample(0, 1), Some(2));
        // Reading unused channel gives zero
        assert_eq!(buffer.read_sample(1, 1), Some(0));
        // write and read an active channel
        assert_eq!(buffer.write_sample(0, 1, &25), Some(false));
        assert_eq!(buffer.read_sample(0, 1), Some(25));
        // write to an unused channel is successful (but does nothing)
        assert_eq!(buffer.write_sample(1, 1, &26), Some(false));
        // reading outside the actual size gives None
        assert_eq!(buffer.read_sample(0, 10), None);
        assert_eq!(buffer.read_sample(1, 10), None);
        assert_eq!(buffer.read_sample(2, 1), None);
        // RMS of the active channel should be 14.55
        assert!((buffer.channel_rms(0) - 14.5).abs() < 0.1);
        // RMS of the unused channel should be zero
        assert_eq!(buffer.channel_rms(1), 0.0);
    }

    use crate::tests::check_copy_within;

    #[test]
    fn copy_within_interleaved_slice() {
        let mut data = vec![0; 20];
        let mut adapter = InterleavedSlice::new_mut(&mut data, 2, 10).unwrap();
        check_copy_within(&mut adapter);
    }

    #[test]
    fn copy_within_sequential_slice() {
        let mut data = vec![0; 20];
        let mut adapter = SequentialSlice::new_mut(&mut data, 2, 10).unwrap();
        check_copy_within(&mut adapter);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn copy_within_interleaved_vecs() {
        let mut data = vec![vec![0; 2]; 10];
        let mut adapter = InterleavedSliceOfVecs::new_mut(&mut data, 2, 10).unwrap();
        check_copy_within(&mut adapter);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn copy_within_sequential_vecs() {
        let mut data = vec![vec![0; 10]; 2];
        let mut adapter = SequentialSliceOfVecs::new_mut(&mut data, 2, 10).unwrap();
        check_copy_within(&mut adapter);
    }

    #[test]
    fn copy_within_sequential_slices() {
        let mut data_ch_0 = vec![0; 10];
        let mut data_ch_1 = vec![0; 10];
        let mut data = vec![data_ch_0.as_mut_slice(), data_ch_1.as_mut_slice()];

        let mut adapter = SequentialSliceOfSlices::new_mut(&mut data, 2, 10).unwrap();
        check_copy_within(&mut adapter);
    }

    #[test]
    fn test_interleaved_slice_with_generic_tester() {
        let mut data = vec![0usize; 8];
        let mut buffer = InterleavedSlice::new_mut(&mut data, 2, 4).unwrap();
        test_adapter_mut_methods(&mut buffer);
    }

    #[test]
    fn test_sequential_slice_with_generic_tester() {
        let mut data = vec![0usize; 8];
        let mut buffer = SequentialSlice::new_mut(&mut data, 2, 4).unwrap();
        test_adapter_mut_methods(&mut buffer);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_interleaved_slice_of_vecs_with_generic_tester() {
        let mut data = vec![vec![0usize; 2]; 4];
        let mut buffer = InterleavedSliceOfVecs::new_mut(&mut data, 2, 4).unwrap();
        test_adapter_mut_methods(&mut buffer);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_sequential_slice_of_vecs_with_generic_tester() {
        let mut data = vec![vec![0usize; 4]; 2];
        let mut buffer = SequentialSliceOfVecs::new_mut(&mut data, 2, 4).unwrap();
        test_adapter_mut_methods(&mut buffer);
    }

    #[test]
    fn test_sequential_slice_of_slices_with_generic_tester() {
        let mut data_ch_0 = vec![0usize; 4];
        let mut data_ch_1 = vec![0usize; 4];
        let mut data = vec![data_ch_0.as_mut_slice(), data_ch_1.as_mut_slice()];

        let mut buffer = SequentialSliceOfSlices::new_mut(&mut data, 2, 4).unwrap();
        test_adapter_mut_methods(&mut buffer);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_sparse_sequential_slice_of_vecs_with_generic_tester() {
        let mut data = vec![vec![0usize; 4], vec![0usize; 4]];
        let mask = [true, true];
        let mut buffer = SparseSequentialSliceOfVecs::new_mut(&mut data, 2, 4, &mask).unwrap();
        test_adapter_mut_methods(&mut buffer);
    }

    #[test]
    fn test_sparse_sequential_slice_of_slices_with_generic_tester() {
        let mut data_ch_0 = vec![0usize; 4];
        let mut data_ch_1 = vec![0usize; 4];
        let mut data = vec![data_ch_0.as_mut_slice(), data_ch_1.as_mut_slice()];

        let mask = [true, true];
        let mut buffer = SparseSequentialSliceOfSlices::new_mut(&mut data, 2, 4, &mask).unwrap();
        test_adapter_mut_methods(&mut buffer);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn size_error_kind_for_too_few_channels_in_sequential_vecs() {
        // Sequential layout expects one Vec per channel.
        // Here we provide only 1 channel Vec, but request 2 channels.
        // Frame length is valid (3), so the only wrong dimension is channel count.
        let data = vec![vec![0_i32; 3]];
        let err = match SequentialSliceOfVecs::new(&data, 2, 3) {
            Ok(_) => panic!("expected Err for too few channels"),
            Err(err) => err,
        };
        assert!(
            matches!(err, SizeError::ChannelsContainer { .. }),
            "expected SizeError::ChannelsContainer for too few channels, got {err:?}"
        );
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn size_error_kind_for_too_few_channels_in_sparse_sequential_vecs() {
        // Sparse sequential layout still expects the outer slice length to match channels.
        // Here we provide only 1 channel Vec, but request 2 channels.
        // The mask also has length 2, and the active channel's frame length is valid,
        // so the failing dimension is again the number of channels in `data`.
        let data = vec![vec![0_i32; 3]];
        let mask = [true, true];
        let err = match SparseSequentialSliceOfVecs::new(&data, 2, 3, &mask) {
            Ok(_) => panic!("expected Err for too few channels"),
            Err(err) => err,
        };
        assert!(
            matches!(err, SizeError::ChannelsContainer { .. }),
            "expected SizeError::ChannelsContainer for too few channels, got {err:?}"
        );
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn size_error_kind_for_too_few_frames_in_interleaved_vecs() {
        // Interleaved layout expects one Vec per frame.
        // Here we provide only 1 frame Vec, but request 2 frames.
        // Each frame Vec has valid channel length (2), so the only wrong dimension is frame count.
        let data = vec![vec![0_i32; 2]];
        let err = match InterleavedSliceOfVecs::new(&data, 2, 2) {
            Ok(_) => panic!("expected Err for too few frames"),
            Err(err) => err,
        };
        assert!(
            matches!(err, SizeError::FramesContainer { .. }),
            "expected SizeError::FramesContainer for too few frames in interleaved input, got {err:?}"
        );
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn size_error_kind_for_short_inner_channel_buffer_in_sequential_vecs() {
        // Two channel buffers are present, so outer channel count is valid.
        // Channel index 1 is too short for the requested frame count.
        let data = vec![vec![0_i32; 3], vec![0_i32; 2]];
        let err = match SequentialSliceOfVecs::new(&data, 2, 3) {
            Ok(_) => panic!("expected Err for short inner channel buffer"),
            Err(err) => err,
        };
        assert!(
            matches!(
                err,
                SizeError::ChannelBuffer {
                    index: 1,
                    actual: 2,
                    required: 3
                }
            ),
            "expected SizeError::ChannelBuffer at index 1, got {err:?}"
        );
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn size_error_kind_for_short_inner_frame_buffer_in_interleaved_vecs() {
        // Two frame buffers are present, so outer frame count is valid.
        // Frame index 1 is too short for the requested channel count.
        let data = vec![vec![0_i32; 2], vec![0_i32; 1]];
        let err = match InterleavedSliceOfVecs::new(&data, 2, 2) {
            Ok(_) => panic!("expected Err for short inner frame buffer"),
            Err(err) => err,
        };
        assert!(
            matches!(
                err,
                SizeError::FrameBuffer {
                    index: 1,
                    actual: 1,
                    required: 2
                }
            ),
            "expected SizeError::FrameBuffer at index 1, got {err:?}"
        );
    }

    #[test]
    fn size_error_kind_for_short_flat_buffer_in_interleaved_slice() {
        // Flat interleaved data requires channels * frames samples.
        // Here we provide 3 samples but request 2 * 2 = 4.
        let data = [0_i32; 3];
        let err = match InterleavedSlice::new(&data, 2, 2) {
            Ok(_) => panic!("expected Err for short flat buffer"),
            Err(err) => err,
        };
        assert!(
            matches!(
                err,
                SizeError::Total {
                    actual: 3,
                    required: 4
                }
            ),
            "expected SizeError::Total with required flat length 4, got {err:?}"
        );
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn size_error_kind_for_wrong_mask_length_in_sparse_sequential_vecs() {
        // Data dimensions are valid, but mask length must match channels.
        let data = vec![vec![0_i32; 3], vec![0_i32; 3]];
        let mask = [true];
        let err = match SparseSequentialSliceOfVecs::new(&data, 2, 3, &mask) {
            Ok(_) => panic!("expected Err for wrong mask length"),
            Err(err) => err,
        };
        assert!(
            matches!(
                err,
                SizeError::Mask {
                    actual: 1,
                    required: 2
                }
            ),
            "expected SizeError::Mask with required length 2, got {err:?}"
        );
    }

    #[test]
    fn overflowing_dimensions_are_rejected() {
        // channels * frames overflows usize and must not wrap to a small
        // value that passes the length check. See the soundness fix in
        // `check_slice_length!`.
        let data = [0_i32; 4];
        // big * big wraps to exactly 0 (2^usize::BITS), which the unfixed check
        // would treat as "required 0" and accept. Portable across 32/64-bit.
        let big = 1_usize << (usize::BITS / 2);
        assert!(
            matches!(
                InterleavedSlice::new(&data, big, big),
                Err(SizeError::Total { .. })
            ),
            "overflowing channels * frames must be rejected"
        );
        assert!(
            matches!(
                SequentialSlice::new(&data, big, big),
                Err(SizeError::Total { .. })
            ),
            "overflowing channels * frames must be rejected"
        );
    }

    #[test]
    fn copy_frames_within_rejects_overflowing_range() {
        // src + count overflows usize. The guard must not wrap and allow an
        // out-of-bounds copy.
        let mut data = [0_i32; 6];
        let mut buf = InterleavedSlice::new_mut(&mut data, 2, 3).unwrap();
        assert_eq!(buf.copy_frames_within(usize::MAX, 0, 2), None);
        assert_eq!(buf.copy_frames_within(0, usize::MAX, 2), None);
    }
}
