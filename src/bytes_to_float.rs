//! # Converting wrappers for raw bytes
//! This module provides wrappers for slices of bytes.
//! The wrapper enables reading and writing samples from/to the byte slice with
//! on-the-fly format conversion to float.
//!
//! The wrappers implement the traits [crate::Adapter] and [crate::AdapterMut],
//! that provide simple methods for accessing the audio samples of a buffer.
//!
//! ## Data order
//! There are two wrappers availabe for each sample format,
//! one for interleaved and one for sequential data.
//!
//! ## Example
//! Wrap a Vec of bytes as an interleaved buffer of 16-bit little endian
//! integer samples and print all the values.
//! ```
//! use audioadapter::bytes_to_float::InterleavedBytes;
//! use audioadapter::Adapter;
//! use audioadapter::sample::I16LE;
//!
//! // make a vector with some fake data.
//! // 2 channels * 3 frames * 2 bytes per sample => 12 bytes
//! let data: Vec<u8> = vec![0, 0, 0, 128, 0, 64, 0, 192, 0, 32, 0, 224];
//!
//! // wrap the data
//! let buffer: InterleavedBytes<f32, I16LE, _> = InterleavedBytes::new(&data, 2, 3).unwrap();
//!
//! // Loop over all samples and print their values
//! for channel in 0..2 {
//!     for frame in 0..3 {
//!         let value = buffer.read_sample(channel, frame).unwrap();
//!         println!(
//!             "Channel: {}, frame: {}, value: {}",
//!             channel, frame, value
//!         );
//!     }
//! }
//! ```

use crate::sample::BytesSample;
use crate::sample::RawSample;
use crate::SizeError;
use crate::{check_slice_length, implement_size_getters};
use crate::{Adapter, AdapterMut};
use num_traits::float::Float;

macro_rules! implement_read_func {
    () => {
        unsafe fn read_sample_unchecked(&self, channel: usize, frame: usize) -> T {
            let idx = self.calc_index(channel, frame);
            let raw = self.buf.get_unchecked(idx..idx + U::BYTES_PER_SAMPLE);
            let sample = U::from_slice(raw);
            sample.to_scaled_float::<T>()
        }
    };
}

macro_rules! implement_write_func {
    () => {
        unsafe fn write_sample_unchecked(
            &mut self,
            channel: usize,
            frame: usize,
            value: &T,
        ) -> bool {
            let idx = self.calc_index(channel, frame);
            let converted = U::from_scaled_float(*value);
            self.buf[idx..idx + U::BYTES_PER_SAMPLE].copy_from_slice(converted.value.as_slice());
            converted.clipped
        }
    };
}

/// A wrapper for a slice containing interleaved samples stored as raw bytes.
pub struct InterleavedBytes<'a, T, U, V> {
    _phantom: core::marker::PhantomData<&'a T>,
    _phantom_raw: core::marker::PhantomData<&'a U>,
    buf: V,
    frames: usize,
    channels: usize,
}

impl<'a, T, U> InterleavedBytes<'a, T, U, &'a [u8]>
where
    U: BytesSample,
{
    /// Create a new wrapper for a slice containing samples
    /// stored as raw bytes in _interleaved_ order.
    /// The slice length must be at least `mem::size_of(U) * frames * channels`.
    /// It is allowed to be longer than needed,
    /// but these extra values cannot
    /// be accessed via the `Adapter` trait methods.
    pub fn new(buf: &'a [u8], channels: usize, frames: usize) -> Result<Self, SizeError> {
        check_slice_length!(channels, frames, buf.len(), U::BYTES_PER_SAMPLE);
        Ok(Self {
            _phantom: core::marker::PhantomData,
            _phantom_raw: core::marker::PhantomData,
            buf,
            frames,
            channels,
        })
    }
}

/// A wrapper for a mutable slice containing interleaved samples stored as raw bytes.
impl<'a, T, U> InterleavedBytes<'a, T, U, &'a mut [u8]>
where
    U: BytesSample,
{
    /// Create a new wrapper for a mutable slice containing samples
    /// stored as raw bytes in _interleaved_ order.
    /// The slice length must be at least `mem::size_of(U) * frames * channels`.
    /// It is allowed to be longer than needed,
    /// but these extra values cannot
    /// be accessed via the `Adapter` trait methods.
    pub fn new_mut(buf: &'a mut [u8], channels: usize, frames: usize) -> Result<Self, SizeError> {
        check_slice_length!(channels, frames, buf.len(), U::BYTES_PER_SAMPLE);
        Ok(Self {
            _phantom: core::marker::PhantomData,
            _phantom_raw: core::marker::PhantomData,
            buf,
            frames,
            channels,
        })
    }
}

impl<'a, T, U, V> InterleavedBytes<'a, T, U, V>
where
    U: BytesSample,
{
    fn calc_index(&self, channel: usize, frame: usize) -> usize {
        let sample_idx = self.channels * frame + channel;
        sample_idx * U::BYTES_PER_SAMPLE
    }
}

impl<'a, T, U> Adapter<'a, T> for InterleavedBytes<'a, T, U, &'a [u8]>
where
    T: Float + 'a,
    U: BytesSample + RawSample,
{
    implement_size_getters!();

    implement_read_func!();
}

impl<'a, T, U> Adapter<'a, T> for InterleavedBytes<'a, T, U, &'a mut [u8]>
where
    T: Float + 'a,
    U: BytesSample + RawSample,
{
    implement_size_getters!();

    implement_read_func!();
}

impl<'a, T, U> AdapterMut<'a, T> for InterleavedBytes<'a, T, U, &'a mut [u8]>
where
    T: Float + 'a,
    U: BytesSample + RawSample,
{
    implement_write_func!();
}

//   _____         _
//  |_   _|__  ___| |_ ___
//    | |/ _ \/ __| __/ __|
//    | |  __/\__ \ |_\__ \
//    |_|\___||___/\__|___/

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sample::*;

    #[test]
    fn read_i16_newtype() {
        let data: [u8; 12] = [0, 0, 0, 128, 0, 64, 0, 192, 0, 32, 0, 224];
        let buffer: InterleavedBytes<f32, I16LE, _> = InterleavedBytes::new(&data, 2, 3).unwrap();
        assert_eq!(buffer.read_sample(0, 0).unwrap(), 0.0);
        assert_eq!(buffer.read_sample(1, 0).unwrap(), -1.0);
        assert_eq!(buffer.read_sample(0, 1).unwrap(), 0.5);
        assert_eq!(buffer.read_sample(1, 1).unwrap(), -0.5);
        assert_eq!(buffer.read_sample(0, 2).unwrap(), 0.25);
        assert_eq!(buffer.read_sample(1, 2).unwrap(), -0.25);
    }

    #[test]
    fn write_i16_newtype() {
        let expected: [u8; 12] = [0, 0, 0, 128, 0, 64, 0, 192, 0, 32, 0, 224];
        let mut data = [0; 12];
        let mut buffer: InterleavedBytes<f32, I16LE, _> =
            InterleavedBytes::new_mut(&mut data, 2, 3).unwrap();
        buffer.write_sample(0, 0, &0.0).unwrap();
        buffer.write_sample(1, 0, &-1.0).unwrap();
        buffer.write_sample(0, 1, &0.5).unwrap();
        buffer.write_sample(1, 1, &-0.5).unwrap();
        buffer.write_sample(0, 2, &0.25).unwrap();
        buffer.write_sample(1, 2, &-0.25).unwrap();
        assert_eq!(data, expected);
    }

    #[test]
    fn read_i32() {
        let data: [u8; 24] = [
            0, 0, 0, 0, 0, 0, 0, 128, 0, 0, 0, 64, 0, 0, 0, 192, 0, 0, 0, 32, 0, 0, 0, 224,
        ];
        let buffer: InterleavedBytes<f32, I32LE, _> = InterleavedBytes::new(&data, 2, 3).unwrap();
        assert_eq!(buffer.read_sample(0, 0).unwrap(), 0.0);
        assert_eq!(buffer.read_sample(1, 0).unwrap(), -1.0);
        assert_eq!(buffer.read_sample(0, 1).unwrap(), 0.5);
        assert_eq!(buffer.read_sample(1, 1).unwrap(), -0.5);
        assert_eq!(buffer.read_sample(0, 2).unwrap(), 0.25);
        assert_eq!(buffer.read_sample(1, 2).unwrap(), -0.25);
    }

    #[test]
    fn write_i32() {
        let expected: [u8; 24] = [
            0, 0, 0, 0, 0, 0, 0, 128, 0, 0, 0, 64, 0, 0, 0, 192, 0, 0, 0, 32, 0, 0, 0, 224,
        ];
        let mut data = [0; 24];
        let mut buffer: InterleavedBytes<f32, I32LE, _> =
            InterleavedBytes::new_mut(&mut data, 2, 3).unwrap();

        buffer.write_sample(0, 0, &0.0).unwrap();
        buffer.write_sample(1, 0, &-1.0).unwrap();
        buffer.write_sample(0, 1, &0.5).unwrap();
        buffer.write_sample(1, 1, &-0.5).unwrap();
        buffer.write_sample(0, 2, &0.25).unwrap();
        buffer.write_sample(1, 2, &-0.25).unwrap();
        assert_eq!(data, expected);
    }

    #[test]
    fn from_slice_i32() {
        let expected_data: [u8; 24] = [
            0, 0, 0, 0, 0, 0, 0, 128, 0, 0, 0, 64, 0, 0, 0, 192, 0, 0, 0, 32, 0, 0, 0, 224,
        ];
        let values_left = [0.0, 0.5, 0.25];
        let values_right = [-1.0, -0.5, -0.25];
        let mut data = [0; 24];
        let mut buffer: InterleavedBytes<f32, I32LE, _> =
            InterleavedBytes::new_mut(&mut data, 2, 3).unwrap();

        buffer.write_from_slice_to_channel(0, 0, &values_left);
        buffer.write_from_slice_to_channel(1, 0, &values_right);
        assert_eq!(data, expected_data);
    }

    #[test]
    fn to_slice_i32() {
        let data: [u8; 24] = [
            0, 0, 0, 0, 0, 0, 0, 128, 0, 0, 0, 64, 0, 0, 0, 192, 0, 0, 0, 32, 0, 0, 0, 224,
        ];
        let expected_left = [0.0, 0.5, 0.25];
        let expected_right = [-1.0, -0.5, -0.25];
        let mut values_left = [0.0; 3];
        let mut values_right = [0.0; 3];
        let buffer: InterleavedBytes<f32, I32LE, _> = InterleavedBytes::new(&data, 2, 3).unwrap();

        buffer.write_from_channel_to_slice(0, 0, &mut values_left);
        buffer.write_from_channel_to_slice(1, 0, &mut values_right);
        assert_eq!(values_left, expected_left);
        assert_eq!(values_right, expected_right);
    }

    // Check that a buffer is Send + Sync,
    // meaning it can be sent between threads.
    // This test is not designed to be run, only to compile.
    #[allow(dead_code)]
    fn test_adapter_send_and_sync<T: Sync + Send + Clone>() {
        fn is_send<T: Send>() {}
        fn is_sync<T: Sync>() {}
        is_send::<InterleavedBytes<f32, I32LE, &[u8]>>();
        is_sync::<InterleavedBytes<f32, I32LE, &[u8]>>();
    }
}
