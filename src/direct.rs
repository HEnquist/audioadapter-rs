//! # direct wrappers
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
//! Each wrapper exist in an _interleaved_ and _sequential_ version.
//!
//! ### Example
//! Wrap a Vec of i32 as an interleaved buffer
//! and print all the values.
//! ```
//! use audioadapter::direct::InterleavedSlice;
//! use audioadapter::Adapter;
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

use crate::{check_slice_length, implement_size_getters};
use crate::{Adapter, AdapterMut};

#[cfg(feature = "std")]
macro_rules! check_slice_and_vec_length {
    ($buf:expr, $channels:expr, $frames:expr, sequential) => {
        if $buf.len() < $channels {
            return Err(SizeError::Frame {
                index: 0,
                actual: $buf.len(),
                required: $channels,
            });
        }
        for (idx, chan) in $buf.iter().enumerate() {
            if chan.len() < $frames {
                return Err(SizeError::Channel {
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
            return Err(SizeError::Frame {
                index: 0,
                actual: $buf.len(),
                required: $channels,
            });
        }
        for (idx, (chan, active)) in $buf.iter().zip($mask).enumerate() {
            if *active && chan.len() < $frames {
                return Err(SizeError::Channel {
                    index: idx,
                    actual: chan.len(),
                    required: $frames,
                });
            }
        }
    };
    ($buf:expr, $channels:expr, $frames:expr, interleaved) => {
        if $buf.len() < $frames {
            return Err(SizeError::Channel {
                index: 0,
                actual: $buf.len(),
                required: $frames,
            });
        }
        for (idx, frame) in $buf.iter().enumerate() {
            if frame.len() < $channels {
                return Err(SizeError::Frame {
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
#[cfg(feature = "std")]
pub struct SequentialSliceOfVecs<U> {
    buf: U,
    frames: usize,
    channels: usize,
}

#[cfg(feature = "std")]
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

#[cfg(feature = "std")]
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

#[cfg(feature = "std")]
impl<'a, T> Adapter<'a, T> for SequentialSliceOfVecs<&'a [Vec<T>]>
where
    T: Clone,
{
    unsafe fn read_sample_unchecked(&self, channel: usize, frame: usize) -> T {
        self.buf.get_unchecked(channel).get_unchecked(frame).clone()
    }

    implement_size_getters!();

    fn write_from_channel_to_slice(&self, channel: usize, skip: usize, slice: &mut [T]) -> usize {
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

#[cfg(feature = "std")]
impl<'a, T> Adapter<'a, T> for SequentialSliceOfVecs<&'a mut [Vec<T>]>
where
    T: Clone,
{
    unsafe fn read_sample_unchecked(&self, channel: usize, frame: usize) -> T {
        self.buf.get_unchecked(channel).get_unchecked(frame).clone()
    }

    implement_size_getters!();

    fn write_from_channel_to_slice(&self, channel: usize, skip: usize, slice: &mut [T]) -> usize {
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

#[cfg(feature = "std")]
impl<'a, T> AdapterMut<'a, T> for SequentialSliceOfVecs<&'a mut [Vec<T>]>
where
    T: Clone + Copy
{
    unsafe fn write_sample_unchecked(&mut self, channel: usize, frame: usize, value: &T) -> bool {
        *self.buf.get_unchecked_mut(channel).get_unchecked_mut(frame) = value.clone();
        false
    }

    fn write_from_slice_to_channel(
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
        if src + count > self.frames || dest + count > self.frames {
            return None;
        }
        for ch in self.buf.iter_mut() {
            ch.copy_within(src..src+count, dest);
        }
        Some(count)
    }
}

//
// =========================== SparseSequentialSliceOfVecs ===========================
//

/// Wrapper for a slice of length `channels`, containing vectors of length `frames`.
/// Each vector contains the samples for all frames of one channel.
/// This is similar to [SequentialSliceOfVecs],
/// but here vectors for unused channels may be empty.
#[cfg(feature = "std")]
pub struct SparseSequentialSliceOfVecs<U> {
    buf: U,
    frames: usize,
    channels: usize,
    mask: Vec<bool>,
}

#[cfg(feature = "std")]
impl<'a, T> SparseSequentialSliceOfVecs<&'a [Vec<T>]> {
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
        active_channels_mask: &[bool],
    ) -> Result<Self, SizeError> {
        let mask = active_channels_mask.to_vec();
        check_slice_and_vec_length!(buf, channels, frames, &mask, sequential);
        Ok(Self {
            buf,
            frames,
            channels,
            mask,
        })
    }
}

#[cfg(feature = "std")]
impl<'a, T> SparseSequentialSliceOfVecs<&'a mut [Vec<T>]> {
    /// Create a new `SparseSequentialSliceOfVecs` to wrap a mutable slice of vectors.
    /// The slice must contain at least `channels` vectors,
    /// and each vector must be at least `frames` long.
    /// They are allowed to be longer than needed,
    /// but these extra frames or channels cannot
    /// be accessed via the trait methods.
    pub fn new_mut(
        buf: &'a mut [Vec<T>],
        channels: usize,
        frames: usize,
        active_channels_mask: &[bool],
    ) -> Result<Self, SizeError> {
        let mask = active_channels_mask.to_vec();
        check_slice_and_vec_length!(buf, channels, frames, &mask, sequential);
        Ok(Self {
            buf,
            frames,
            channels,
            mask,
        })
    }
}

#[cfg(feature = "std")]
impl<'a, T> Adapter<'a, T> for SparseSequentialSliceOfVecs<&'a [Vec<T>]>
where
    T: Clone,
{
    fn read_sample(&self, channel: usize, frame: usize) -> Option<T> {
        if channel >= self.channels || !self.mask[channel] || frame >= self.frames {
            return None;
        }
        Some(unsafe { self.read_sample_unchecked(channel, frame) })
    }

    unsafe fn read_sample_unchecked(&self, channel: usize, frame: usize) -> T {
        self.buf.get_unchecked(channel).get_unchecked(frame).clone()
    }

    implement_size_getters!();

    fn write_from_channel_to_slice(&self, channel: usize, skip: usize, slice: &mut [T]) -> usize {
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

#[cfg(feature = "std")]
impl<'a, T> Adapter<'a, T> for SparseSequentialSliceOfVecs<&'a mut [Vec<T>]>
where
    T: Clone,
{
    fn read_sample(&self, channel: usize, frame: usize) -> Option<T> {
        if channel >= self.channels || !self.mask[channel] || frame >= self.frames {
            return None;
        }
        Some(unsafe { self.read_sample_unchecked(channel, frame) })
    }

    unsafe fn read_sample_unchecked(&self, channel: usize, frame: usize) -> T {
        self.buf.get_unchecked(channel).get_unchecked(frame).clone()
    }

    implement_size_getters!();

    fn write_from_channel_to_slice(&self, channel: usize, skip: usize, slice: &mut [T]) -> usize {
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

#[cfg(feature = "std")]
impl<'a, T> AdapterMut<'a, T> for SparseSequentialSliceOfVecs<&'a mut [Vec<T>]>
where
    T: Clone + Copy,
{
    fn write_sample(&mut self, channel: usize, frame: usize, value: &T) -> Option<bool> {
        if channel >= self.channels || !self.mask[channel] || frame >= self.frames {
            return None;
        }
        Some(unsafe { self.write_sample_unchecked(channel, frame, value) })
    }

    unsafe fn write_sample_unchecked(&mut self, channel: usize, frame: usize, value: &T) -> bool {
        *self.buf.get_unchecked_mut(channel).get_unchecked_mut(frame) = value.clone();
        false
    }

    fn write_from_slice_to_channel(
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
        if src + count > self.frames || dest + count > self.frames {
            return None;
        }
        for (ch, active) in self.buf.iter_mut().zip(self.mask.iter()) {
            if *active {
                ch.copy_within(src..src+count, dest);
            }
        }
        Some(count)
    }
}

//
// =========================== InterleavedSliceOfVecs ===========================
//

/// Wrapper for a slice of length `frames`, containing vectors of length `channels`.
/// Each vector contains the samples for all channels of one frame.
#[cfg(feature = "std")]
pub struct InterleavedSliceOfVecs<U> {
    buf: U,
    frames: usize,
    channels: usize,
}

#[cfg(feature = "std")]
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

#[cfg(feature = "std")]
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

#[cfg(feature = "std")]
impl<'a, T> Adapter<'a, T> for InterleavedSliceOfVecs<&'a [Vec<T>]>
where
    T: Clone,
{
    unsafe fn read_sample_unchecked(&self, channel: usize, frame: usize) -> T {
        self.buf.get_unchecked(frame).get_unchecked(channel).clone()
    }

    implement_size_getters!();

    fn write_from_frame_to_slice(&self, frame: usize, skip: usize, slice: &mut [T]) -> usize {
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

#[cfg(feature = "std")]
impl<'a, T> Adapter<'a, T> for InterleavedSliceOfVecs<&'a mut [Vec<T>]>
where
    T: Clone,
{
    unsafe fn read_sample_unchecked(&self, channel: usize, frame: usize) -> T {
        self.buf.get_unchecked(frame).get_unchecked(channel).clone()
    }

    implement_size_getters!();

    fn write_from_frame_to_slice(&self, frame: usize, skip: usize, slice: &mut [T]) -> usize {
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

#[cfg(feature = "std")]
impl<'a, T> AdapterMut<'a, T> for InterleavedSliceOfVecs<&'a mut [Vec<T>]>
where
    T: Clone,
{
    unsafe fn write_sample_unchecked(&mut self, channel: usize, frame: usize, value: &T) -> bool {
        *self.buf.get_unchecked_mut(frame).get_unchecked_mut(channel) = value.clone();
        false
    }

    fn write_from_slice_to_frame(
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

    fn copy_frames_within(&mut self, src: usize, dest: usize, count: usize) -> Option<usize> {
        unimplemented!()
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

impl<'a, T> Adapter<'a, T> for InterleavedSlice<&'a [T]>
where
    T: Clone,
{
    unsafe fn read_sample_unchecked(&self, channel: usize, frame: usize) -> T {
        let index = self.calc_index(channel, frame);
        self.buf.get_unchecked(index).clone()
    }

    implement_size_getters!();

    fn write_from_frame_to_slice(&self, frame: usize, skip: usize, slice: &mut [T]) -> usize {
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

impl<'a, T> Adapter<'a, T> for InterleavedSlice<&'a mut [T]>
where
    T: Clone,
{
    unsafe fn read_sample_unchecked(&self, channel: usize, frame: usize) -> T {
        let index = self.calc_index(channel, frame);
        self.buf.get_unchecked(index).clone()
    }

    implement_size_getters!();

    fn write_from_frame_to_slice(&self, frame: usize, skip: usize, slice: &mut [T]) -> usize {
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

impl<'a, T> AdapterMut<'a, T> for InterleavedSlice<&'a mut [T]>
where
    T: Clone + Copy,
{
    unsafe fn write_sample_unchecked(&mut self, channel: usize, frame: usize, value: &T) -> bool {
        let index = self.calc_index(channel, frame);
        *self.buf.get_unchecked_mut(index) = value.clone();
        false
    }

    fn write_from_slice_to_frame(
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
        if src + count > self.frames || dest + count > self.frames {
            return None;
        }
        self.buf.copy_within(src*self.channels..(src+count)*self.channels, dest*self.channels);
        Some(count)
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

impl<'a, T> Adapter<'a, T> for SequentialSlice<&'a [T]>
where
    T: Clone,
{
    unsafe fn read_sample_unchecked(&self, channel: usize, frame: usize) -> T {
        let index = self.calc_index(channel, frame);
        self.buf.get_unchecked(index).clone()
    }

    implement_size_getters!();

    fn write_from_channel_to_slice(&self, channel: usize, skip: usize, slice: &mut [T]) -> usize {
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
impl<'a, T> Adapter<'a, T> for SequentialSlice<&'a mut [T]>
where
    T: Clone,
{
    unsafe fn read_sample_unchecked(&self, channel: usize, frame: usize) -> T {
        let index = self.calc_index(channel, frame);
        self.buf.get_unchecked(index).clone()
    }

    implement_size_getters!();

    fn write_from_channel_to_slice(&self, channel: usize, skip: usize, slice: &mut [T]) -> usize {
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

impl<'a, T> AdapterMut<'a, T> for SequentialSlice<&'a mut [T]>
where
    T: Clone + Copy,
{
    unsafe fn write_sample_unchecked(&mut self, channel: usize, frame: usize, value: &T) -> bool {
        let index = self.calc_index(channel, frame);
        *self.buf.get_unchecked_mut(index) = value.clone();
        false
    }

    fn write_from_slice_to_channel(
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
        if src + count > self.frames || dest + count > self.frames {
            return None;
        }
        for ch in 0..self.channels {
            let offset = ch*self.frames;
            self.buf.copy_within(src + offset..src+offset+count, dest+offset);
        }
        Some(count)
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
        buffer.write_from_channel_to_slice(0, 1, &mut other1);
        buffer.write_from_channel_to_slice(1, 0, &mut other2);
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
        buffer.write_from_frame_to_slice(0, 1, &mut other1);
        buffer.write_from_frame_to_slice(1, 0, &mut other2);
        assert_eq!(other1[0], 4);
        assert_eq!(other2[0], 2);
        assert_eq!(other2[1], 5);
        assert_eq!(other2[2], 0);
    }

    fn test_mut_slice_channel(buffer: &mut dyn AdapterMut<i32>) {
        insert_data(buffer);
        let other1 = [8, 9];
        let other2 = [10, 11, 12, 13];
        buffer.write_from_slice_to_channel(0, 1, &other1);
        buffer.write_from_slice_to_channel(1, 0, &other2);
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
        buffer.write_from_slice_to_frame(0, 0, &other1);
        buffer.write_from_slice_to_frame(1, 0, &other2);
        assert_eq!(buffer.read_sample(0, 0).unwrap(), 8);
        assert_eq!(buffer.read_sample(1, 0).unwrap(), 4);
        assert_eq!(buffer.read_sample(0, 1).unwrap(), 10);
        assert_eq!(buffer.read_sample(1, 1).unwrap(), 11);
        assert_eq!(buffer.read_sample(0, 2).unwrap(), 3);
        assert_eq!(buffer.read_sample(1, 2).unwrap(), 6);
    }

    #[cfg(feature = "std")]
    #[test]
    fn vec_of_channels() {
        let mut data = vec![vec![0_i32; 3], vec![0_i32; 3]];
        let mut buffer = SequentialSliceOfVecs::new_mut(&mut data, 2, 3).unwrap();
        test_get(&mut buffer);
        test_slice_channel(&mut buffer);
        test_slice_frame(&mut buffer);
        test_mut_slice_channel(&mut buffer);
        test_mut_slice_frame(&mut buffer);
    }

    #[cfg(feature = "std")]
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
    #[cfg(feature = "std")]
    #[test]
    fn boxed_buffer() {
        let mut data = [1_i32, 2, 3, 4, 5, 6];
        let boxed: Box<dyn Adapter<i32>> = Box::new(SequentialSlice::new(&mut data, 2, 3).unwrap());
        assert_eq!(boxed.read_sample(0, 0).unwrap(), 1);
    }

    // Check that a buffer is Send + Sync,
    // meaning it can be sent between threads.
    // This test is not designed to be run, only to compile.
    #[allow(dead_code)]
    fn test_adapter_send_and_sync<T: Sync + Send + Clone>() {
        fn is_send<T: Send>() {}
        fn is_sync<T: Sync>() {}
        is_send::<InterleavedSlice<f32>>();
        is_sync::<InterleavedSlice<f32>>();
        #[cfg(feature = "std")]
        is_send::<InterleavedSliceOfVecs<f32>>();
        #[cfg(feature = "std")]
        is_sync::<InterleavedSliceOfVecs<f32>>();
    }

    #[test]
    fn copy_channel_from_other() {
        let data_other = [1.0_f32, 2.0, 3.0, 4.0, 5.0, 6.0];
        let other = SequentialSlice::new(&data_other, 2, 3).unwrap();
        let mut data = [0.0; 6];
        let mut buffer = SequentialSlice::new_mut(&mut data, 2, 3).unwrap();
        // copy second and third element of second channel of other to first and second element of first channel
        let res1 = buffer.write_from_other_to_channel(&other, 1, 0, 1, 0, 2);
        // copy first and second element of first channel of other to second and third element of second channel
        let res2 = buffer.write_from_other_to_channel(&other, 0, 1, 0, 1, 2);
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

    #[cfg(feature = "std")]
    #[test]
    fn sparse_sequential() {
        let mut data = vec![vec![1, 2, 3], Vec::new()];
        let mask = vec![true, false];
        let mut buffer = SparseSequentialSliceOfVecs::new_mut(&mut data, 2, 3, &mask).unwrap();
        assert_eq!(buffer.read_sample(0, 1), Some(2));
        assert_eq!(buffer.read_sample(1, 1), None);
        assert_eq!(buffer.write_sample(0, 1, &25), Some(false));
        assert_eq!(buffer.write_sample(1, 1, &26), None);
        assert_eq!(buffer.read_sample(0, 1), Some(25));
    }
}
