//! # owning wrappers
//!
//! This module is a collection of wrappers that own the sample data.
//!
//! ## Available wrappers
//! Wrappers are available for vectors, `Vec<T>`,
//! with samples stored in _interleaved_ and _sequential_ order.
//!
//! ### Example
//! Wrap a `Vec<i32>` as an interleaved buffer
//! and print all the values.
//! ```
//! use audioadapter::owned::InterleavedOwned;
//! use audioadapter::Adapter;
//!
//! // make a vector with some dummy data.
//! // 2 channels * 3 frames => 6 samples
//! let data: Vec<i32> = vec![1, 2, 3, 4, 5, 6];
//!
//! // wrap the data
//! let buffer = InterleavedOwned::new_from(data, 2, 3).unwrap();
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
//!
//! // Take back the vector
//! let _data = buffer.take_data();
//! ```
//!

use crate::SizeError;

use crate::{check_slice_length, implement_size_getters};
use crate::{Adapter, AdapterMut};

//
// =========================== InterleavedOwned ===========================
//

/// Wrapper for a vector of length `frames * channels`.
/// The samples are stored in _interleaved_ order,
/// where all the samples for one frame are stored consecutively,
/// followed by the samples for the next frame.
/// For a stereo buffer containing four frames, the order is
/// `L1, R1, L2, R2, L3, R3, L4, R4`
pub struct InterleavedOwned<U> {
    buf: Vec<U>,
    frames: usize,
    channels: usize,
}

impl<U> InterleavedOwned<U> {
    fn calc_index(&self, channel: usize, frame: usize) -> usize {
        frame * self.channels + channel
    }
}

impl<'a, T> InterleavedOwned<T>
where
    T: Clone + 'a,
{
    /// Create a new `InterleavedOwned` by allocaing a new vector filled with `value`.
    pub fn new(value: T, channels: usize, frames: usize) -> Self {
        let buf = vec![value; channels * frames];
        Self {
            buf,
            frames,
            channels,
        }
    }

    /// Create a new `InterleavedOwned` by taking ownership of an existing vector.
    /// The vector length must be at least `frames*channels`.
    /// It is allowed to be longer than needed,
    /// but these extra values cannot be accessed via the trait methods.
    pub fn new_from(buf: Vec<T>, channels: usize, frames: usize) -> Result<Self, SizeError> {
        check_slice_length!(channels, frames, buf.len());
        Ok(Self {
            buf,
            frames,
            channels,
        })
    }

    /// Take ownership of the data from the `InterleavedOwned`.
    pub fn take_data(self) -> Vec<T> {
        self.buf
    }
}

impl<'a, T> Adapter<'a, T> for InterleavedOwned<T>
where
    T: Clone + 'a,
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

impl<'a, T> AdapterMut<'a, T> for InterleavedOwned<T>
where
    T: Clone + Copy + 'a,
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
// =========================== SequentialOwned ===========================
//

/// Wrapper for a vector of length `frames * channels`.
/// The samples are stored in _sequential_ order,
/// where all the samples for one channel are stored consecutively,
/// followed by the samples for the next channel.
/// For a stereo buffer containing four frames, the order is
/// `L1, L2, L3, L4, R1, R2, R3, R4`
pub struct SequentialOwned<U> {
    buf: Vec<U>,
    frames: usize,
    channels: usize,
}

impl<U> SequentialOwned<U> {
    fn calc_index(&self, channel: usize, frame: usize) -> usize {
        channel * self.frames + frame
    }
}

impl<'a, T> SequentialOwned<T>
where
    T: Clone + 'a,
{
    /// Create a new `SequentialOwned` by allocaing a new vector filled with `value`.
    pub fn new(value: T, channels: usize, frames: usize) -> Self {
        let buf = vec![value; channels * frames];
        Self {
            buf,
            frames,
            channels,
        }
    }

    /// Create a new `SequentialOwned` by taking ownership of an existing vector.
    /// The vector length must be at least `frames*channels`.
    /// It is allowed to be longer than needed,
    /// but these extra values cannot be accessed via the trait methods.
    pub fn new_from(buf: Vec<T>, channels: usize, frames: usize) -> Result<Self, SizeError> {
        check_slice_length!(channels, frames, buf.len());
        Ok(Self {
            buf,
            frames,
            channels,
        })
    }

    /// Take ownership of the data from the `SequentialOwned`.
    pub fn take_data(self) -> Vec<T> {
        self.buf
    }
}

impl<'a, T> Adapter<'a, T> for SequentialOwned<T>
where
    T: Clone + 'a,
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

impl<'a, T> AdapterMut<'a, T> for SequentialOwned<T>
where
    T: Clone + Copy + 'a,
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
        buffer.write_sample(0, 0, &1);
        buffer.write_sample(0, 1, &2);
        buffer.write_sample(0, 2, &3);
        buffer.write_sample(1, 0, &4);
        buffer.write_sample(1, 1, &5);
        buffer.write_sample(1, 2, &6);
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

    #[test]
    fn interleaved() {
        let data = vec![1_i32, 4, 2, 5, 3, 6];
        let mut buffer = InterleavedOwned::new_from(data, 2, 3).unwrap();
        test_get(&mut buffer);
        test_slice_channel(&mut buffer);
        test_slice_frame(&mut buffer);
        test_mut_slice_channel(&mut buffer);
        test_mut_slice_frame(&mut buffer);
        // get the inner vector
        let _data = buffer.take_data();
    }

    #[test]
    fn sequential() {
        let data = vec![1_i32, 2, 3, 4, 5, 6];
        let mut buffer = SequentialOwned::new_from(data, 2, 3).unwrap();
        test_get(&mut buffer);
        test_slice_channel(&mut buffer);
        test_slice_frame(&mut buffer);
        test_mut_slice_channel(&mut buffer);
        test_mut_slice_frame(&mut buffer);
        // get the inner vector
        let _data = buffer.take_data();
    }

    // This tests that an Adapter is object safe.
    #[cfg(feature = "std")]
    #[test]
    fn boxed_buffer() {
        let boxed: Box<dyn Adapter<i32>> = Box::new(SequentialOwned::new(1, 2, 3));
        assert_eq!(boxed.read_sample(0, 0).unwrap(), 1);
    }

    // Check that a buffer is Send + Sync,
    // meaning it can be sent between threads.
    // This test is not designed to be run, only to compile.
    #[allow(dead_code)]
    fn test_adapter_send_and_sync<T: Sync + Send + Clone>() {
        fn is_send<T: Send>() {}
        fn is_sync<T: Sync>() {}
        is_send::<InterleavedOwned<f32>>();
        is_sync::<InterleavedOwned<f32>>();
    }

    #[test]
    fn copy_channel_from_other() {
        let data_other = vec![1.0_f32, 2.0, 3.0, 4.0, 5.0, 6.0];
        let other = SequentialOwned::new_from(data_other, 2, 3).unwrap();
        let mut buffer: SequentialOwned<f32> = SequentialOwned::new(0.0, 2, 3);
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
        let mut buffer = InterleavedOwned::new(1, 2, 3);
        buffer.fill_channel_with(0, &2);
        let expected: [i32; 6] = [2, 1, 2, 1, 2, 1];
        let data = buffer.take_data();
        assert_eq!(data, expected);
    }

    #[test]
    fn fill_frame() {
        let mut buffer = InterleavedOwned::new(1, 2, 3);
        buffer.fill_frame_with(1, &2);
        let expected: [i32; 6] = [1, 1, 2, 2, 1, 1];
        let data = buffer.take_data();
        assert_eq!(data, expected);
    }

    #[test]
    fn fill_buffer() {
        let mut buffer = InterleavedOwned::new(1, 2, 3);
        buffer.fill_with(&2);
        let expected: [i32; 6] = [2; 6];
        let data = buffer.take_data();
        assert_eq!(data, expected);
    }
}
