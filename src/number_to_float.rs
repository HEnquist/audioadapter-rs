//! # Converting wrappers for numerical values
//! This module provides wrappers for slices of numbers.
//! The wrapper enables reading and writing samples from/to the slice with
//! on-the-fly format conversion between the original type and float.
//!
//! ## Data order
//! There are two wrappers availabe for each sample format,
//! one for interleaved and one for sequential data.
//!
//! ## Example
//! Wrap a Vec of 16-bit integer samples as an interleaved buffer
//! and print all the values.
//! ```
//! use audioadapter::number_to_float::InterleavedNumbers;
//! use audioadapter::Adapter;
//!
//! // make a vector with some data.
//! // 2 channels * 3 frames => 6 samples
//! let data: Vec<i16> = vec![1, 2, 3, 4, 5, 6];
//!
//! // wrap the data
//! let buffer: InterleavedNumbers<&[i16], f32> = InterleavedNumbers::new(&data, 2, 3).unwrap();
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
//!
//! ## Example with raw bytes
//! Wrap a Vec of bytes as an interleaved buffer of 16-bit little endian
//! integer samples and print all the values.
//! ```
//! use audioadapter::number_to_float::InterleavedNumbers;
//! use audioadapter::Adapter;
//! use audioadapter::sample::I16LE;
//!
//! // make a vector with some dummy data.
//! // 2 channels * 3 frames * 2 bytes per sample => 12 bytes
//! let data: Vec<u8> = vec![0, 0, 0, 128, 0, 64, 0, 192, 0, 32, 0, 224];
//!
//! // wrap the data
//! let buffer: InterleavedNumbers<&[I16LE], f32> = InterleavedNumbers::new_from_bytes(&data, 2, 3).unwrap();
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
use core::mem::size_of;

use num_traits::Float;

use crate::sample::RawSample;
use crate::SizeError;
use crate::{check_slice_length, implement_size_getters};
use crate::{Adapter, AdapterMut};

/// A macro for creating a view of an immutable slice of bytes
/// as a different type.
#[macro_export]
macro_rules! byte_slice_as_type {
    ($slice:ident, $type:ty) => {
        unsafe {
            let ptr = $slice.as_ptr() as *const $type;
            let len = $slice.len();
            core::slice::from_raw_parts(ptr, len / core::mem::size_of::<$type>())
        }
    };
}

/// A macro for creating a view of a mutable slice of bytes
/// as a different type.
#[macro_export]
macro_rules! byte_slice_as_type_mut {
    ($slice:ident, $type:ty) => {
        unsafe {
            let ptr = $slice.as_mut_ptr() as *mut $type;
            let len = $slice.len();
            core::slice::from_raw_parts_mut(ptr, len / core::mem::size_of::<$type>())
        }
    };
}

/// A wrapper for a slice containing interleaved numerical samples.
pub struct InterleavedNumbers<U, V> {
    _phantom: core::marker::PhantomData<V>,
    buf: U,
    frames: usize,
    channels: usize,
}

/// A wrapper for a slice containing interleaved numerical samples.
pub struct SequentialNumbers<U, V> {
    _phantom: core::marker::PhantomData<V>,
    buf: U,
    frames: usize,
    channels: usize,
}

impl<U, V> InterleavedNumbers<U, V> {
    fn calc_index(&self, channel: usize, frame: usize) -> usize {
        frame * self.channels + channel
    }
}

impl<U, V> SequentialNumbers<U, V> {
    fn calc_index(&self, channel: usize, frame: usize) -> usize {
        frame + channel * self.frames
    }
}

impl<'a, U, T> InterleavedNumbers<&'a [U], T>
where
    T: 'a,
{
    /// Create a new wrapper for an immutable slice
    /// of numerical samples implementing [RawSample],
    /// stored in _interleaved_ order.
    /// The slice length must be at least `frames*channels`.
    /// It is allowed to be longer than needed,
    /// but these extra values cannot
    /// be accessed via the `Adapter` trait methods.
    pub fn new(buf: &'a [U], channels: usize, frames: usize) -> Result<Self, SizeError> {
        check_slice_length!(channels, frames, buf.len());
        Ok(Self {
            _phantom: core::marker::PhantomData,
            buf,
            frames,
            channels,
        })
    }

    /// Create a new wrapper for a mutable slice
    /// of numerical samples implementing [RawSample],
    /// stored as raw bytes in _interleaved_ order.
    /// The slice length must be at least `core::mem::size_of::<U>() * frames * channels`.
    /// It is allowed to be longer than needed,
    /// but these extra values cannot
    /// be accessed via the `Adapter` trait methods.
    pub fn new_from_bytes(
        buf: &'a [u8],
        channels: usize,
        frames: usize,
    ) -> Result<Self, SizeError> {
        check_slice_length!(channels, frames, buf.len(), size_of::<U>());
        let buf_view = byte_slice_as_type!(buf, U);
        Ok(Self {
            _phantom: core::marker::PhantomData,
            buf: buf_view,
            frames,
            channels,
        })
    }
}

impl<'a, U, T> InterleavedNumbers<&'a mut [U], T>
where
    T: 'a,
{
    /// Create a new wrapper for a mutable slice
    /// of numerical samples implementing [RawSample],
    /// stored in _interleaved_ order.
    /// The slice length must be at least `frames*channels`.
    /// It is allowed to be longer than needed,
    /// but these extra values cannot
    /// be accessed via the `Adapter` or `AdapterMut` trait methods.
    pub fn new_mut(buf: &'a mut [U], channels: usize, frames: usize) -> Result<Self, SizeError> {
        check_slice_length!(channels, frames, buf.len());
        Ok(Self {
            _phantom: core::marker::PhantomData,
            buf,
            frames,
            channels,
        })
    }

    /// Create a new wrapper for a mutable slice
    /// of numerical samples implementing [RawSample],
    /// stored as raw bytes in _interleaved_ order.
    /// The slice length must be at least `core::mem::size_of::<U>() * frames * channels`.
    /// It is allowed to be longer than needed,
    /// but these extra values cannot
    /// be accessed via the `Adapter` trait methods.
    pub fn new_from_bytes_mut(
        buf: &'a mut [u8],
        channels: usize,
        frames: usize,
    ) -> Result<Self, SizeError> {
        check_slice_length!(channels, frames, buf.len(), size_of::<U>());
        let buf_view = byte_slice_as_type_mut!(buf, U);
        Ok(Self {
            _phantom: core::marker::PhantomData,
            buf: buf_view,
            frames,
            channels,
        })
    }
}

impl<'a, U, T> SequentialNumbers<&'a [U], T>
where
    T: 'a,
{
    /// Create a new wrapper for an immutable slice
    /// of numerical samples implementing [RawSample],
    /// stored in _sequential_ order.
    /// The slice length must be at least `frames*channels`.
    /// It is allowed to be longer than needed,
    /// but these extra values cannot
    /// be accessed via the `Adapter` trait methods.
    pub fn new(buf: &'a [U], channels: usize, frames: usize) -> Result<Self, SizeError> {
        check_slice_length!(channels, frames, buf.len());
        Ok(Self {
            _phantom: core::marker::PhantomData,
            buf,
            frames,
            channels,
        })
    }

    /// Create a new wrapper for a mutable slice
    /// of numerical samples implementing [RawSample],
    /// stored as raw bytes in _sequential_ order.
    /// The slice length must be at least `core::mem::size_of::<U>() * frames * channels`.
    /// It is allowed to be longer than needed,
    /// but these extra values cannot
    /// be accessed via the `Adapter` trait methods.
    pub fn new_from_bytes(
        buf: &'a [u8],
        channels: usize,
        frames: usize,
    ) -> Result<Self, SizeError> {
        check_slice_length!(channels, frames, buf.len(), size_of::<U>());
        let buf_view = byte_slice_as_type!(buf, U);
        Ok(Self {
            _phantom: core::marker::PhantomData,
            buf: buf_view,
            frames,
            channels,
        })
    }
}

impl<'a, U, T> SequentialNumbers<&'a mut [U], T>
where
    T: 'a,
{
    /// Create a new wrapper for a mutable slice
    /// of numerical samples implementing [RawSample],
    /// stored in _sequential_ order.
    /// The slice length must be at least `frames*channels`.
    /// It is allowed to be longer than needed,
    /// but these extra values cannot
    /// be accessed via the `Adapter` or `AdapterMut` trait methods.
    pub fn new_mut(buf: &'a mut [U], channels: usize, frames: usize) -> Result<Self, SizeError> {
        check_slice_length!(channels, frames, buf.len());
        Ok(Self {
            _phantom: core::marker::PhantomData,
            buf,
            frames,
            channels,
        })
    }

    /// Create a new wrapper for a mutable slice
    /// of numerical samples implementing [RawSample],
    /// stored as raw bytes in _sequential_ order.
    /// The slice length must be at least `core::mem::size_of::<U>() * frames * channels`.
    /// It is allowed to be longer than needed,
    /// but these extra values cannot
    /// be accessed via the `Adapter` trait methods.
    pub fn new_from_bytes_mut(
        buf: &'a mut [u8],
        channels: usize,
        frames: usize,
    ) -> Result<Self, SizeError> {
        check_slice_length!(channels, frames, buf.len(), size_of::<U>());
        let buf_view = byte_slice_as_type_mut!(buf, U);
        Ok(Self {
            _phantom: core::marker::PhantomData,
            buf: buf_view,
            frames,
            channels,
        })
    }
}

macro_rules! impl_traits_newtype {
    ($order:ident) => {
        paste::item! {
            impl<'a, T, U> Adapter<'a, T> for [< $order Numbers >]<&'a [U], T>
            where
                T: Float + 'a,
                U: RawSample,
            {
                unsafe fn read_sample_unchecked(&self, channel: usize, frame: usize) -> T {
                    let index = self.calc_index(channel, frame);
                    self.buf[index].to_scaled_float()
                }

                implement_size_getters!();
            }

            impl<'a, T, U> Adapter<'a, T> for [< $order Numbers >]<&'a mut [U], T>
            where
                T: Float + 'a,
                U: RawSample,
            {
                unsafe fn read_sample_unchecked(&self, channel: usize, frame: usize) -> T {
                    let index = self.calc_index(channel, frame);
                    self.buf[index].to_scaled_float()
                }

                implement_size_getters!();
            }

            impl<'a, T, U> AdapterMut<'a, T> for [< $order Numbers >]<&'a mut [U], T>
            where
                T: Float + 'a,
                U: RawSample,
            {
                unsafe fn write_sample_unchecked(&mut self, channel: usize, frame: usize, value: &T) -> bool {
                    let index = self.calc_index(channel, frame);
                    let converted = U::from_scaled_float(*value);
                    self.buf[index] = converted.value;
                    converted.clipped
                }
            }
        }
    };
}

impl_traits_newtype!(Interleaved);
impl_traits_newtype!(Sequential);

//   _____         _
//  |_   _|__  ___| |_ ___
//    | |/ _ \/ __| __/ __|
//    | |  __/\__ \ |_\__ \
//    |_|\___||___/\__|___/

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sample::{I16LE, I24LE};

    #[test]
    fn read_i32() {
        let data: [i32; 6] = [0, -2 << 30, 2 << 29, -2 << 29, 2 << 28, -2 << 28];
        let buffer: InterleavedNumbers<&[i32], f32> = InterleavedNumbers::new(&data, 2, 3).unwrap();
        assert_eq!(buffer.read_sample(0, 0).unwrap(), 0.0);
        assert_eq!(buffer.read_sample(1, 0).unwrap(), -1.0);
        assert_eq!(buffer.read_sample(0, 1).unwrap(), 0.5);
        assert_eq!(buffer.read_sample(1, 1).unwrap(), -0.5);
        assert_eq!(buffer.read_sample(0, 2).unwrap(), 0.25);
        assert_eq!(buffer.read_sample(1, 2).unwrap(), -0.25);
    }

    #[test]
    fn read_i16() {
        let data: [i16; 6] = [0, -2 << 14, 2 << 13, -2 << 13, 2 << 12, -2 << 12];
        let buffer: InterleavedNumbers<&[i16], f32> = InterleavedNumbers::new(&data, 2, 3).unwrap();
        assert_eq!(buffer.read_sample(0, 0).unwrap(), 0.0);
        assert_eq!(buffer.read_sample(1, 0).unwrap(), -1.0);
        assert_eq!(buffer.read_sample(0, 1).unwrap(), 0.5);
        assert_eq!(buffer.read_sample(1, 1).unwrap(), -0.5);
        assert_eq!(buffer.read_sample(0, 2).unwrap(), 0.25);
        assert_eq!(buffer.read_sample(1, 2).unwrap(), -0.25);
    }

    #[test]
    fn read_i8() {
        let data: [i8; 6] = [0, -2 << 6, 2 << 5, -2 << 5, 2 << 4, -2 << 4];
        let buffer: InterleavedNumbers<&[i8], f32> = InterleavedNumbers::new(&data, 2, 3).unwrap();
        assert_eq!(buffer.read_sample(0, 0).unwrap(), 0.0);
        assert_eq!(buffer.read_sample(1, 0).unwrap(), -1.0);
        assert_eq!(buffer.read_sample(0, 1).unwrap(), 0.5);
        assert_eq!(buffer.read_sample(1, 1).unwrap(), -0.5);
        assert_eq!(buffer.read_sample(0, 2).unwrap(), 0.25);
        assert_eq!(buffer.read_sample(1, 2).unwrap(), -0.25);
    }

    #[test]
    fn read_u8() {
        let data: [u8; 6] = [
            128,
            128 - (2 << 6),
            128 + (2 << 5),
            128 - (2 << 5),
            128 + (2 << 4),
            128 - (2 << 4),
        ];
        let buffer: InterleavedNumbers<&[u8], f32> = InterleavedNumbers::new(&data, 2, 3).unwrap();
        assert_eq!(buffer.read_sample(0, 0).unwrap(), 0.0);
        assert_eq!(buffer.read_sample(1, 0).unwrap(), -1.0);
        assert_eq!(buffer.read_sample(0, 1).unwrap(), 0.5);
        assert_eq!(buffer.read_sample(1, 1).unwrap(), -0.5);
        assert_eq!(buffer.read_sample(0, 2).unwrap(), 0.25);
        assert_eq!(buffer.read_sample(1, 2).unwrap(), -0.25);
    }

    #[test]
    fn write_i32() {
        let expected: [i32; 6] = [0, -2 << 30, 2 << 29, -2 << 29, 2 << 28, -2 << 28];
        let mut data = [0; 6];
        let mut buffer: InterleavedNumbers<&mut [i32], f32> =
            InterleavedNumbers::new_mut(&mut data, 2, 3).unwrap();

        buffer.write_sample(0, 0, &0.0).unwrap();
        buffer.write_sample(1, 0, &-1.0).unwrap();
        buffer.write_sample(0, 1, &0.5).unwrap();
        buffer.write_sample(1, 1, &-0.5).unwrap();
        buffer.write_sample(0, 2, &0.25).unwrap();
        buffer.write_sample(1, 2, &-0.25).unwrap();
        assert_eq!(data, expected);
    }

    #[test]
    fn write_i16() {
        let expected: [i16; 6] = [0, -2 << 14, 2 << 13, -2 << 13, 2 << 12, -2 << 12];
        let mut data = [0; 6];
        let mut buffer: InterleavedNumbers<&mut [i16], f32> =
            InterleavedNumbers::new_mut(&mut data, 2, 3).unwrap();

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
        let expected_data: [i32; 6] = [0, -2 << 30, 2 << 29, -2 << 29, 2 << 28, -2 << 28];
        let values_left = [0.0, 0.5, 0.25];
        let values_right = [-1.0, -0.5, -0.25];
        let mut data = [0; 6];
        let mut buffer: InterleavedNumbers<&mut [i32], f32> =
            InterleavedNumbers::new_mut(&mut data, 2, 3).unwrap();

        buffer.write_from_slice_to_channel(0, 0, &values_left);
        buffer.write_from_slice_to_channel(1, 0, &values_right);
        assert_eq!(data, expected_data);
    }

    #[test]
    fn to_slice_i32() {
        let data: [i32; 6] = [0, -2 << 30, 2 << 29, -2 << 29, 2 << 28, -2 << 28];
        let expected_left = [0.0, 0.5, 0.25];
        let expected_right = [-1.0, -0.5, -0.25];
        let mut values_left = [0.0; 3];
        let mut values_right = [0.0; 3];
        let buffer: InterleavedNumbers<&[i32], f32> = InterleavedNumbers::new(&data, 2, 3).unwrap();

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
        is_send::<InterleavedNumbers<&[i32], f32>>();
        is_sync::<InterleavedNumbers<&[i32], f32>>();
    }

    #[test]
    fn read_i16_bytes_interleaved() {
        let data: [u8; 12] = [0, 0, 0, 128, 0, 64, 0, 192, 0, 32, 0, 224];
        let buffer: InterleavedNumbers<&[I16LE], f32> =
            InterleavedNumbers::new_from_bytes(&data, 2, 3).unwrap();
        assert_eq!(buffer.read_sample(0, 0).unwrap(), 0.0);
        assert_eq!(buffer.read_sample(1, 0).unwrap(), -1.0);
        assert_eq!(buffer.read_sample(0, 1).unwrap(), 0.5);
        assert_eq!(buffer.read_sample(1, 1).unwrap(), -0.5);
        assert_eq!(buffer.read_sample(0, 2).unwrap(), 0.25);
        assert_eq!(buffer.read_sample(1, 2).unwrap(), -0.25);
    }

    #[test]
    fn write_i16_bytes_interleaved() {
        let expected: [u8; 12] = [0, 0, 0, 128, 0, 64, 0, 192, 0, 32, 0, 224];
        let mut data = [0; 12];
        let mut buffer: InterleavedNumbers<&mut [I16LE], f32> =
            InterleavedNumbers::new_from_bytes_mut(&mut data, 2, 3).unwrap();
        buffer.write_sample(0, 0, &0.0).unwrap();
        buffer.write_sample(1, 0, &-1.0).unwrap();
        buffer.write_sample(0, 1, &0.5).unwrap();
        buffer.write_sample(1, 1, &-0.5).unwrap();
        buffer.write_sample(0, 2, &0.25).unwrap();
        buffer.write_sample(1, 2, &-0.25).unwrap();
        assert_eq!(data, expected);
    }

    #[test]
    fn read_i24_bytes_interleaved() {
        let data: [u8; 18] = [0, 0, 0, 0, 0, 128, 0, 0, 64, 0, 0, 192, 0, 0, 32, 0, 0, 224];
        let buffer: InterleavedNumbers<&[I24LE<[u8; 3]>], f32> =
            InterleavedNumbers::new_from_bytes(&data, 2, 3).unwrap();
        assert_eq!(buffer.read_sample(0, 0).unwrap(), 0.0);
        assert_eq!(buffer.read_sample(1, 0).unwrap(), -1.0);
        assert_eq!(buffer.read_sample(0, 1).unwrap(), 0.5);
        assert_eq!(buffer.read_sample(1, 1).unwrap(), -0.5);
        assert_eq!(buffer.read_sample(0, 2).unwrap(), 0.25);
        assert_eq!(buffer.read_sample(1, 2).unwrap(), -0.25);
    }
}
