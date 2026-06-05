//! # Converting wrappers for existing `audioadapter` buffers
//!
//! This module provides wrappers for buffers
//! that already implement the `audioadapter` traits.
//! The wrappers enable reading and writing samples from/to another buffer
//! with on-the-fly format conversion.
//!
//! ## Example
//! Wrap a `Vec<i16>` as an interleaved buffer,
//! then wrap this again with a converter,
//! and finally read and print all the values as floats.
//! ```
//! use audioadapter::Adapter;
//! use audioadapter_buffers::adapter_to_float::ConvertNumbers;
//! use audioadapter_buffers::direct::InterleavedSlice;
//!
//! // Make a vector with some dummy data.
//! let data: Vec<i16> = vec![1, 2, 3, 4, 5, 6];
//!
//! // Wrap the data as an interleaved i16 buffer.
//! let int_buffer = InterleavedSlice::new(&data, 2, 3).unwrap();
//!
//! // Wrap this buffer with a converter to read the values as floats.
//! let converter = ConvertNumbers::<_, f32>::new(&int_buffer as &dyn Adapter<i16>);
//!
//! // Loop over all samples and print their values
//! for channel in 0..2 {
//!     for frame in 0..3 {
//!         let value = converter.read_sample(channel, frame).unwrap();
//!         println!(
//!             "Channel: {}, frame: {}, value: {}",
//!             channel, frame, value
//!         );
//!     }
//! }
//! ```

use num_traits::{ToPrimitive, float::FloatCore};

use audioadapter::{Adapter, AdapterMut};
use audioadapter_sample::sample::*;

macro_rules! implement_wrapped_size_getters {
    () => {
        fn channels(&self) -> usize {
            self.buf.channels()
        }

        fn frames(&self) -> usize {
            self.buf.frames()
        }
    };
}

/// A wrapper for an [Adapter] or [AdapterMut] buffer containing samples
/// stored as byte arrays.
/// The wrapper enables reading and writing the samples as floats.
///
/// # Type parameters
/// - `T`: the floating point type that samples are converted to and from when
///   reading and writing, for example `f32` or `f64`.
/// - `U`: the byte-wrapper sample format type, for example [`I16_LE`] or
///   [`F32_LE`]. It implements [BytesSample] and [RawSample], and determines the
///   sample size as `[u8; U::BYTES_PER_SAMPLE]`.
/// - `V`: the wrapped buffer type, an [Adapter] or [AdapterMut] over byte arrays
///   of length `U::BYTES_PER_SAMPLE`.
pub struct ConvertBytes<T, U, V>
where
    T: FloatCore + ToPrimitive,
    U: BytesSample + RawSample,
{
    _phantom: core::marker::PhantomData<T>,
    _phantom_raw: core::marker::PhantomData<U>,
    buf: V,
}

macro_rules! byte_convert_traits_newtype {
    ($typename:ident) => {
        impl<'a, T> ConvertBytes<T, $typename, &'a dyn Adapter<[u8; $typename::BYTES_PER_SAMPLE]>>
            where
                T: FloatCore + ToPrimitive,
            {
                #[doc = concat!(
                    "Create a new wrapper for an [Adapter] buffer of byte arrays, `[u8; ",
                    stringify!($typename), "::BYTES_PER_SAMPLE]`, containing samples of type [`",
                    stringify!($typename), "`]."
                )]
                pub fn new(
                    buf: &'a dyn Adapter<[u8; $typename::BYTES_PER_SAMPLE]>,
                ) -> Self {
                    Self {
                        _phantom: core::marker::PhantomData,
                        _phantom_raw: core::marker::PhantomData,
                        buf,
                    }
                }
            }

            impl<'a, T> ConvertBytes<T, $typename, &'a mut dyn AdapterMut<[u8; $typename::BYTES_PER_SAMPLE]>>
            where
                T: FloatCore + ToPrimitive,
            {
                #[doc = concat!(
                    "Create a new wrapper for a mutable [AdapterMut] buffer of byte arrays, `[u8; ",
                    stringify!($typename), "::BYTES_PER_SAMPLE]`, containing samples of type [`",
                    stringify!($typename), "`]."
                )]
                pub fn new_mut(
                    buf: &'a mut dyn AdapterMut<[u8; $typename::BYTES_PER_SAMPLE]>,
                ) -> Self {
                    Self {
                        _phantom: core::marker::PhantomData,
                        _phantom_raw: core::marker::PhantomData,
                        buf,
                    }
                }
            }

            unsafe impl<'a, T> Adapter<T> for ConvertBytes<T, $typename, &'a dyn Adapter<[u8; $typename::BYTES_PER_SAMPLE]>>
            where
            T: FloatCore + ToPrimitive,
            {
                unsafe fn read_sample_unchecked(&self, channel: usize, frame: usize) -> T {
                    let raw = unsafe { self.buf.read_sample_unchecked(channel, frame) };
                    let sample = $typename::from_slice(&raw);
                    sample.to_scaled_float::<T>()
                }

                implement_wrapped_size_getters!();
            }

            unsafe impl<'a, T> Adapter<T> for ConvertBytes<T, $typename, &'a mut dyn AdapterMut<[u8; $typename::BYTES_PER_SAMPLE]>>
            where
            T: FloatCore + ToPrimitive,
            {
                unsafe fn read_sample_unchecked(&self, channel: usize, frame: usize) -> T {
                    let raw = unsafe { self.buf.read_sample_unchecked(channel, frame) };
                    let sample = $typename::from_slice(&raw);
                    sample.to_scaled_float::<T>()
                }

                implement_wrapped_size_getters!();
            }

            unsafe impl<'a, T> AdapterMut<T> for ConvertBytes<T, $typename, &'a mut dyn AdapterMut<[u8; $typename::BYTES_PER_SAMPLE]>>
            where
            T: FloatCore + ToPrimitive,
            {
                unsafe fn write_sample_unchecked(&mut self, channel: usize, frame: usize, value: &T) -> bool {
                    let converted = $typename::from_scaled_float(*value);
                    unsafe {
                        self.buf.write_sample_unchecked(channel, frame, converted.value.as_slice().try_into().unwrap());
                    }
                    converted.clipped
                }

                fn copy_frames_within(&mut self, src: usize, dest: usize, count: usize) -> Option<usize> {
                    self.buf.copy_frames_within(src, dest, count)
                }

                fn copy_sample_within(
                        &mut self,
                        source_channel: usize,
                        source_frame: usize,
                        target_channel: usize,
                        target_frame: usize,
                    ) -> bool {
                    self.buf.copy_sample_within(source_channel, source_frame, target_channel, target_frame)
                }

                fn swap_samples(
                    &mut self,
                    channel_a: usize,
                    frame_a: usize,
                    channel_b: usize,
                    frame_b: usize,
                ) -> bool {
                    self.buf.swap_samples(channel_a, frame_a, channel_b, frame_b)
                }

            }
        }
}

byte_convert_traits_newtype!(I16_LE);
byte_convert_traits_newtype!(I16_BE);
byte_convert_traits_newtype!(U16_LE);
byte_convert_traits_newtype!(U16_BE);
byte_convert_traits_newtype!(I24_LE);
byte_convert_traits_newtype!(I24_BE);
byte_convert_traits_newtype!(U24_LE);
byte_convert_traits_newtype!(U24_BE);
byte_convert_traits_newtype!(I24_4LJ_LE);
byte_convert_traits_newtype!(I24_4LJ_BE);
byte_convert_traits_newtype!(I24_4RJ_LE);
byte_convert_traits_newtype!(I24_4RJ_BE);
byte_convert_traits_newtype!(U24_4LJ_LE);
byte_convert_traits_newtype!(U24_4LJ_BE);
byte_convert_traits_newtype!(U24_4RJ_LE);
byte_convert_traits_newtype!(U24_4RJ_BE);
byte_convert_traits_newtype!(I32_LE);
byte_convert_traits_newtype!(I32_BE);
byte_convert_traits_newtype!(U32_LE);
byte_convert_traits_newtype!(U32_BE);
byte_convert_traits_newtype!(F32_LE);
byte_convert_traits_newtype!(F32_BE);
byte_convert_traits_newtype!(F64_LE);
byte_convert_traits_newtype!(F64_BE);

/// A wrapper for an [Adapter] or [AdapterMut] buffer containing samples
/// stored as numeric types.
/// The wrapper enables reading and writing the samples as floats.
///
/// # Type parameters
/// - `U`: the wrapped buffer type, an [Adapter] or [AdapterMut] over a numeric
///   sample type, for example `&dyn Adapter<i16>`. The sample type implements
///   [RawSample].
/// - `V`: the floating point type that samples are converted to and from when
///   reading and writing, for example `f32` or `f64`.
pub struct ConvertNumbers<U, V> {
    _phantom: core::marker::PhantomData<V>,
    buf: U,
}

impl<'a, T, U> ConvertNumbers<&'a dyn Adapter<U>, T>
where
    T: FloatCore + ToPrimitive,
    U: RawSample,
{
    /// Create a new wrapper for a buffer implementing the [Adapter] trait,
    /// containing numerical samples.
    pub fn new(buf: &'a dyn Adapter<U>) -> Self {
        Self {
            _phantom: core::marker::PhantomData,
            buf,
        }
    }
}

impl<'a, T, U> ConvertNumbers<&'a mut dyn AdapterMut<U>, T>
where
    T: FloatCore + ToPrimitive,
    U: RawSample,
{
    /// Create a new wrapper for a mutable buffer implementing the [AdapterMut] trait,
    /// containing numerical samples.
    pub fn new_mut(buf: &'a mut dyn AdapterMut<U>) -> Self {
        Self {
            _phantom: core::marker::PhantomData,
            buf,
        }
    }
}

unsafe impl<T, U> Adapter<T> for ConvertNumbers<&dyn Adapter<U>, T>
where
    T: FloatCore + ToPrimitive,
    U: RawSample,
{
    unsafe fn read_sample_unchecked(&self, channel: usize, frame: usize) -> T {
        unsafe {
            self.buf
                .read_sample_unchecked(channel, frame)
                .to_scaled_float()
        }
    }

    implement_wrapped_size_getters!();
}

unsafe impl<T, U> Adapter<T> for ConvertNumbers<&mut dyn AdapterMut<U>, T>
where
    T: FloatCore + ToPrimitive,
    U: RawSample,
{
    unsafe fn read_sample_unchecked(&self, channel: usize, frame: usize) -> T {
        unsafe {
            self.buf
                .read_sample_unchecked(channel, frame)
                .to_scaled_float()
        }
    }

    implement_wrapped_size_getters!();
}

unsafe impl<T, U> AdapterMut<T> for ConvertNumbers<&mut dyn AdapterMut<U>, T>
where
    T: FloatCore + ToPrimitive,
    U: RawSample + Clone,
{
    unsafe fn write_sample_unchecked(&mut self, channel: usize, frame: usize, value: &T) -> bool {
        let converted = U::from_scaled_float(*value);
        unsafe {
            self.buf
                .write_sample_unchecked(channel, frame, &converted.value);
        }
        converted.clipped
    }

    fn copy_frames_within(&mut self, src: usize, dest: usize, count: usize) -> Option<usize> {
        self.buf.copy_frames_within(src, dest, count)
    }

    fn copy_sample_within(
        &mut self,
        source_channel: usize,
        source_frame: usize,
        target_channel: usize,
        target_frame: usize,
    ) -> bool {
        self.buf
            .copy_sample_within(source_channel, source_frame, target_channel, target_frame)
    }

    fn swap_samples(
        &mut self,
        channel_a: usize,
        frame_a: usize,
        channel_b: usize,
        frame_b: usize,
    ) -> bool {
        self.buf
            .swap_samples(channel_a, frame_a, channel_b, frame_b)
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
    use crate::direct::InterleavedSlice;
    use audioadapter::Adapter;
    use audioadapter::tests::test_float_adapter_mut_methods;

    #[test]
    fn read_i16_bytes() {
        let data: [[u8; 2]; 6] = [[0, 0], [0, 128], [0, 64], [0, 192], [0, 32], [0, 224]];
        let buffer: InterleavedSlice<&[[u8; 2]]> = InterleavedSlice::new(&data, 2, 3).unwrap();
        let converter: ConvertBytes<f32, I16_LE, _> =
            ConvertBytes::<f32, I16_LE, _>::new(&buffer as &dyn Adapter<[u8; 2]>);
        assert_eq!(converter.read_sample(0, 0).unwrap(), 0.0);
        assert_eq!(converter.read_sample(1, 0).unwrap(), -1.0);
        assert_eq!(converter.read_sample(0, 1).unwrap(), 0.5);
        assert_eq!(converter.read_sample(1, 1).unwrap(), -0.5);
        assert_eq!(converter.read_sample(0, 2).unwrap(), 0.25);
        assert_eq!(converter.read_sample(1, 2).unwrap(), -0.25);
    }

    #[test]
    fn read_i16() {
        let data: [i16; 6] = [0, i16::MIN, 1 << 14, -(1 << 14), 1 << 13, -(1 << 13)];
        let buffer: InterleavedSlice<&[i16]> = InterleavedSlice::new(&data, 2, 3).unwrap();
        let converter: ConvertNumbers<&dyn Adapter<i16>, f32> =
            ConvertNumbers::new(&buffer as &dyn Adapter<i16>);
        assert_eq!(converter.read_sample(0, 0).unwrap(), 0.0);
        assert_eq!(converter.read_sample(1, 0).unwrap(), -1.0);
        assert_eq!(converter.read_sample(0, 1).unwrap(), 0.5);
        assert_eq!(converter.read_sample(1, 1).unwrap(), -0.5);
        assert_eq!(converter.read_sample(0, 2).unwrap(), 0.25);
        assert_eq!(converter.read_sample(1, 2).unwrap(), -0.25);
    }

    #[test]
    fn write_i16_bytes() {
        let expected: [[u8; 2]; 6] = [[0, 0], [0, 128], [0, 64], [0, 192], [0, 32], [0, 224]];
        let mut data = [[0, 0]; 6];
        let mut buffer: InterleavedSlice<&mut [[u8; 2]]> =
            InterleavedSlice::new_mut(&mut data, 2, 3).unwrap();
        let mut converter: ConvertBytes<f32, I16_LE, _> =
            ConvertBytes::<f32, I16_LE, _>::new_mut(&mut buffer as &mut dyn AdapterMut<[u8; 2]>);
        converter.write_sample(0, 0, &0.0).unwrap();
        converter.write_sample(1, 0, &-1.0).unwrap();
        converter.write_sample(0, 1, &0.5).unwrap();
        converter.write_sample(1, 1, &-0.5).unwrap();
        converter.write_sample(0, 2, &0.25).unwrap();
        converter.write_sample(1, 2, &-0.25).unwrap();
        assert_eq!(data, expected);
    }

    #[test]
    fn write_i16() {
        let expected: [i16; 6] = [0, i16::MIN, 1 << 14, -(1 << 14), 1 << 13, -(1 << 13)];
        let mut data = [0; 6];
        let mut buffer: InterleavedSlice<&mut [i16]> =
            InterleavedSlice::new_mut(&mut data, 2, 3).unwrap();
        let mut converter: ConvertNumbers<&mut dyn AdapterMut<i16>, f32> =
            ConvertNumbers::new_mut(&mut buffer as &mut dyn AdapterMut<i16>);
        converter.write_sample(0, 0, &0.0).unwrap();
        converter.write_sample(1, 0, &-1.0).unwrap();
        converter.write_sample(0, 1, &0.5).unwrap();
        converter.write_sample(1, 1, &-0.5).unwrap();
        converter.write_sample(0, 2, &0.25).unwrap();
        converter.write_sample(1, 2, &-0.25).unwrap();
        assert_eq!(data, expected);
    }

    #[test]
    fn test_sequential_owned_with_generic_tester() {
        let mut data = [0; 8];
        let mut buffer: InterleavedSlice<&mut [i32]> =
            InterleavedSlice::new_mut(&mut data, 2, 4).unwrap();
        let mut converter: ConvertNumbers<&mut dyn AdapterMut<i32>, f32> =
            ConvertNumbers::new_mut(&mut buffer as &mut dyn AdapterMut<i32>);
        test_float_adapter_mut_methods(&mut converter);
    }
}
