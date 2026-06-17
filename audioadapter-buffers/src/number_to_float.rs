//! # Converting wrappers for numerical values
//!
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
//! use audioadapter_buffers::number_to_float::InterleavedNumbers;
//! use audioadapter::Adapter;
//!
//! // make a vector with some data.
//! // 2 channels * 3 frames => 6 samples
//! let data: Vec<i16> = vec![1, 2, 3, 4, 5, 6];
//!
//! // wrap the data
//! let buffer = InterleavedNumbers::<_, f32>::new(&data, 2, 3).unwrap();
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
//! use audioadapter_buffers::number_to_float::InterleavedNumbers;
//! use audioadapter::Adapter;
//! use audioadapter_sample::sample::I16_LE;
//!
//! // make a vector with some dummy data.
//! // 2 channels * 3 frames * 2 bytes per sample => 12 bytes
//! let data: Vec<u8> = vec![0, 0, 0, 128, 0, 64, 0, 192, 0, 32, 0, 224];
//!
//! // wrap the data
//! let buffer = InterleavedNumbers::<&[I16_LE], f32>::new_from_bytes(&data, 2, 3).unwrap();
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

use num_traits::{ToPrimitive, float::FloatCore};

use crate::SizeError;
use crate::slicetools::copy_within_slice;
use crate::{check_slice_length, implement_size_getters};
use audioadapter::{Adapter, AdapterMut};
use audioadapter_sample::sample::{
    BytesSample, F32_BE, F32_LE, F64_BE, F64_LE, I16_BE, I16_LE, I24_4LJ_BE, I24_4LJ_LE,
    I24_4RJ_BE, I24_4RJ_LE, I24_BE, I24_LE, I32_BE, I32_LE, I64_BE, I64_LE, RawSample, U16_BE,
    U16_LE, U24_4LJ_BE, U24_4LJ_LE, U24_4RJ_BE, U24_4RJ_LE, U24_BE, U24_LE, U32_BE, U32_LE, U64_BE,
    U64_LE,
};

/// A macro for creating a view of an immutable slice of bytes
/// as a different type.
///
/// This is **not** exported: it transmutes `&[u8]` into `&[$type]`, which is
/// only sound for types with alignment 1 and no validity invariants. Callers
/// must therefore require `$type: PlainBytes`, whose unsafe contract guarantees
/// exactly those properties.
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
///
/// This is **not** exported: it transmutes `&mut [u8]` into `&mut [$type]`,
/// which is only sound for types with alignment 1 and no validity invariants.
/// Callers must therefore require `$type: PlainBytes`, whose unsafe contract
/// guarantees exactly those properties.
macro_rules! byte_slice_as_type_mut {
    ($slice:ident, $type:ty) => {
        unsafe {
            let ptr = $slice.as_mut_ptr() as *mut $type;
            let len = $slice.len();
            core::slice::from_raw_parts_mut(ptr, len / core::mem::size_of::<$type>())
        }
    };
}

/// Marker trait for sample types whose raw byte representation can be viewed
/// directly as the type, enabling the byte-based constructors.
///
/// # Safety
/// The byte-based constructors
/// ([`InterleavedNumbers::new_from_bytes`], [`InterleavedNumbers::new_from_bytes_mut`],
/// [`SequentialNumbers::new_from_bytes`], [`SequentialNumbers::new_from_bytes_mut`])
/// reinterpret a `&[u8]` as a `&[Self]` without copying. For that to be sound,
/// every implementor must guarantee that:
///
/// - `Self` has an alignment of 1, and
/// - every bit pattern of the matching size is a valid value of `Self`, i.e. it
///   is a plain "bag of bytes" with no validity invariants and no padding.
///
/// All the byte-wrapper sample types in [`audioadapter_sample`] are
/// `[u8; N]` newtypes that satisfy both requirements, and this trait is
/// implemented for all of them. If you implement [BytesSample] for your own
/// type and want to use it with those constructors, implement this trait too,
/// upholding the requirements above.
pub unsafe trait PlainBytes: BytesSample {}

macro_rules! impl_plainbytes {
    ($($t:ty),* $(,)?) => {
        $(
            // SAFETY: each type is a `#[derive(..)]` newtype around `[u8; N]`,
            // which has alignment 1 and is valid for every bit pattern.
            unsafe impl PlainBytes for $t {}
        )*
    };
}

impl_plainbytes!(
    I16_LE, I16_BE, U16_LE, U16_BE, I24_LE, I24_BE, U24_LE, U24_BE, I24_4LJ_LE, I24_4LJ_BE,
    I24_4RJ_LE, I24_4RJ_BE, U24_4LJ_LE, U24_4LJ_BE, U24_4RJ_LE, U24_4RJ_BE, I32_LE, I32_BE, U32_LE,
    U32_BE, I64_LE, I64_BE, U64_LE, U64_BE, F32_LE, F32_BE, F64_LE, F64_BE,
);

/// A wrapper for a slice containing interleaved numerical samples.
///
/// # Type parameters
/// - `U`: the wrapped slice type holding the samples, for example `&[i16]`,
///   `&mut [i16]`, or `&[I16_LE]` for the byte-based constructors. The element
///   type implements [RawSample] (and [PlainBytes] when constructing from bytes).
/// - `V`: the floating point type that samples are converted to and from when
///   reading and writing, for example `f32` or `f64`.
pub struct InterleavedNumbers<U, V> {
    _phantom: core::marker::PhantomData<V>,
    buf: U,
    frames: usize,
    channels: usize,
}

/// A wrapper for a slice containing sequential numerical samples.
///
/// # Type parameters
/// - `U`: the wrapped slice type holding the samples, for example `&[i16]`,
///   `&mut [i16]`, or `&[I16_LE]` for the byte-based constructors. The element
///   type implements [RawSample] (and [PlainBytes] when constructing from bytes).
/// - `V`: the floating point type that samples are converted to and from when
///   reading and writing, for example `f32` or `f64`.
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

    /// Create a new wrapper for an immutable slice
    /// of numerical samples implementing [PlainBytes],
    /// stored as raw bytes in _interleaved_ order.
    /// The slice length must be at least `core::mem::size_of::<U>() * frames * channels`.
    /// It is allowed to be longer than needed,
    /// but these extra values cannot
    /// be accessed via the `Adapter` trait methods.
    pub fn new_from_bytes(buf: &'a [u8], channels: usize, frames: usize) -> Result<Self, SizeError>
    where
        U: PlainBytes,
    {
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
    U: Clone,
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
    /// of numerical samples implementing [PlainBytes],
    /// stored as raw bytes in _interleaved_ order.
    /// The slice length must be at least `core::mem::size_of::<U>() * frames * channels`.
    /// It is allowed to be longer than needed,
    /// but these extra values cannot
    /// be accessed via the `Adapter` trait methods.
    pub fn new_from_bytes_mut(
        buf: &'a mut [u8],
        channels: usize,
        frames: usize,
    ) -> Result<Self, SizeError>
    where
        U: PlainBytes,
    {
        check_slice_length!(channels, frames, buf.len(), size_of::<U>());
        let buf_view = byte_slice_as_type_mut!(buf, U);
        Ok(Self {
            _phantom: core::marker::PhantomData,
            buf: buf_view,
            frames,
            channels,
        })
    }

    fn copy_frames_within_impl(&mut self, src: usize, dest: usize, count: usize) -> Option<usize> {
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

    /// Create a new wrapper for an immutable slice
    /// of numerical samples implementing [PlainBytes],
    /// stored as raw bytes in _sequential_ order.
    /// The slice length must be at least `core::mem::size_of::<U>() * frames * channels`.
    /// It is allowed to be longer than needed,
    /// but these extra values cannot
    /// be accessed via the `Adapter` trait methods.
    pub fn new_from_bytes(buf: &'a [u8], channels: usize, frames: usize) -> Result<Self, SizeError>
    where
        U: PlainBytes,
    {
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
    U: Clone,
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
    /// of numerical samples implementing [PlainBytes],
    /// stored as raw bytes in _sequential_ order.
    /// The slice length must be at least `core::mem::size_of::<U>() * frames * channels`.
    /// It is allowed to be longer than needed,
    /// but these extra values cannot
    /// be accessed via the `Adapter` trait methods.
    pub fn new_from_bytes_mut(
        buf: &'a mut [u8],
        channels: usize,
        frames: usize,
    ) -> Result<Self, SizeError>
    where
        U: PlainBytes,
    {
        check_slice_length!(channels, frames, buf.len(), size_of::<U>());
        let buf_view = byte_slice_as_type_mut!(buf, U);
        Ok(Self {
            _phantom: core::marker::PhantomData,
            buf: buf_view,
            frames,
            channels,
        })
    }

    fn copy_frames_within_impl(&mut self, src: usize, dest: usize, count: usize) -> Option<usize> {
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
}

macro_rules! impl_traits_newtype {
    ($structname:ident) => {
        unsafe impl<'a, T, U> Adapter<T> for $structname<&'a [U], T>
        where
            T: FloatCore + ToPrimitive + 'a,
            U: RawSample,
        {
            unsafe fn read_sample_unchecked(&self, channel: usize, frame: usize) -> T {
                let index = self.calc_index(channel, frame);
                self.buf[index].to_scaled_float()
            }

            implement_size_getters!();
        }

        unsafe impl<'a, T, U> Adapter<T> for $structname<&'a mut [U], T>
        where
            T: FloatCore + ToPrimitive + 'a,
            U: RawSample,
        {
            unsafe fn read_sample_unchecked(&self, channel: usize, frame: usize) -> T {
                let index = self.calc_index(channel, frame);
                self.buf[index].to_scaled_float()
            }

            implement_size_getters!();
        }

        unsafe impl<'a, T, U> AdapterMut<T> for $structname<&'a mut [U], T>
        where
            T: FloatCore + ToPrimitive + 'a,
            U: RawSample + Clone,
        {
            unsafe fn write_sample_unchecked(
                &mut self,
                channel: usize,
                frame: usize,
                value: &T,
            ) -> bool {
                let index = self.calc_index(channel, frame);
                let converted = U::from_scaled_float(*value);
                self.buf[index] = converted.value;
                converted.clipped
            }

            fn copy_frames_within(
                &mut self,
                src: usize,
                dest: usize,
                count: usize,
            ) -> Option<usize> {
                self.copy_frames_within_impl(src, dest, count)
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
    };
}

impl_traits_newtype!(InterleavedNumbers);
impl_traits_newtype!(SequentialNumbers);

//   _____         _
//  |_   _|__  ___| |_ ___
//    | |/ _ \/ __| __/ __|
//    | |  __/\__ \ |_\__ \
//    |_|\___||___/\__|___/

#[cfg(test)]
mod tests {
    use super::*;
    use audioadapter_sample::sample::{I16_LE, I24_LE};

    #[test]
    fn read_i32() {
        let data: [i32; 6] = [0, -2 << 30, 2 << 29, -2 << 29, 2 << 28, -2 << 28];
        let buffer = InterleavedNumbers::<_, f32>::new(&data, 2, 3).unwrap();
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
        let buffer = InterleavedNumbers::<_, f32>::new(&data, 2, 3).unwrap();
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
        let buffer = InterleavedNumbers::<_, f32>::new(&data, 2, 3).unwrap();
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
        let buffer = InterleavedNumbers::<_, f32>::new(&data, 2, 3).unwrap();
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
        let mut buffer = InterleavedNumbers::<_, f32>::new_mut(&mut data, 2, 3).unwrap();

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
        let mut buffer = InterleavedNumbers::<_, f32>::new_mut(&mut data, 2, 3).unwrap();

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
        let mut buffer = InterleavedNumbers::<_, f32>::new_mut(&mut data, 2, 3).unwrap();

        buffer.copy_from_slice_to_channel(0, 0, &values_left);
        buffer.copy_from_slice_to_channel(1, 0, &values_right);
        assert_eq!(data, expected_data);
    }

    #[test]
    fn to_slice_i32() {
        let data: [i32; 6] = [0, -2 << 30, 2 << 29, -2 << 29, 2 << 28, -2 << 28];
        let expected_left = [0.0, 0.5, 0.25];
        let expected_right = [-1.0, -0.5, -0.25];
        let mut values_left = [0.0; 3];
        let mut values_right = [0.0; 3];
        let buffer: InterleavedNumbers<_, f32> = InterleavedNumbers::new(&data, 2, 3).unwrap();

        buffer.copy_from_channel_to_slice(0, 0, &mut values_left);
        buffer.copy_from_channel_to_slice(1, 0, &mut values_right);
        assert_eq!(values_left, expected_left);
        assert_eq!(values_right, expected_right);
    }

    // Check that a buffer is Send + Sync,
    // meaning it can be sent between threads.
    // This test is not designed to be run, only to compile.
    #[allow(dead_code)]
    fn test_adapter_send_and_sync() {
        fn is_send<T: Send>() {}
        fn is_sync<T: Sync>() {}
        is_send::<InterleavedNumbers<&[i32], f32>>();
        is_sync::<InterleavedNumbers<&[i32], f32>>();
    }

    #[test]
    fn read_i16_bytes_interleaved() {
        let data: [u8; 12] = [0, 0, 0, 128, 0, 64, 0, 192, 0, 32, 0, 224];
        let buffer = InterleavedNumbers::<&[I16_LE], f32>::new_from_bytes(&data, 2, 3).unwrap();
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
        let mut buffer =
            InterleavedNumbers::<&mut [I16_LE], f32>::new_from_bytes_mut(&mut data, 2, 3).unwrap();
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
        let buffer = InterleavedNumbers::<&[I24_LE], f32>::new_from_bytes(&data, 2, 3).unwrap();
        assert_eq!(buffer.read_sample(0, 0).unwrap(), 0.0);
        assert_eq!(buffer.read_sample(1, 0).unwrap(), -1.0);
        assert_eq!(buffer.read_sample(0, 1).unwrap(), 0.5);
        assert_eq!(buffer.read_sample(1, 1).unwrap(), -0.5);
        assert_eq!(buffer.read_sample(0, 2).unwrap(), 0.25);
        assert_eq!(buffer.read_sample(1, 2).unwrap(), -0.25);
    }
}
