//! # Converting wrappers for existing `audioadapter` buffers
//! This module provides wrappers buffers
//! that already implement the `audioadapter` traits.
//! The wrapper enables reading and writing samples from/to another buffer
//! with on-the-fly format conversion.
//!
//! The wrappers implement the traits [crate::Adapter] and [crate::AdapterMut],
//! that provide simple methods for accessing the audio samples of a buffer.
//!
//! ### Example
//! Wrap a Vec of i16 as an interleaved buffer,
//! then wrap this again with a converter,
//! and finally read and print all the values as floats.
//! ```
//! use audioadapter::direct::InterleavedSlice;
//! use audioadapter::Adapter;
//! use audioadapter::converter::ConvertI16;
//!
//! // Make a vector with some fake data.
//! let data: Vec<i16> = vec![1, 2, 3, 4, 5, 6];
//!
//! // Wrap the data as an interleaved i16 buffer.
//! let int_buffer: InterleavedSlice<&[i16]> = InterleavedSlice::new(&data, 2, 3).unwrap();
//!
//! // Wrap this buffer with a converter to read the values as floats.
//! let converter: ConvertI16<&dyn Adapter<i16>, f32> = ConvertI16::new(&int_buffer as &dyn Adapter<i16>);
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

use crate::{Adapter, AdapterMut};
use rawsample::BytesSample;
use rawsample::NumericSample;

macro_rules! byte_convert_structs {
    ($bytes:expr, $typename:ident) => {
        paste::item! {
            #[doc = "A wrapper for an [Adapter] or [AdapterMut] buffer containing `" $typename "` samples"]
            #[doc = " stored as byte arrays, `[u8; " $bytes "]`"]
            pub struct [< Convert $typename >]<U, V> {
                _phantom: core::marker::PhantomData<V>,
                buf: U,
            }
        }
    };
}

macro_rules! byte_convert_traits {
    ($read_func:ident, $write_func:ident, $bytes:expr, $typename:ident) => {
        paste::item! {

            impl<'a, T> [< Convert $typename >]<&'a dyn Adapter<'a, [u8; $bytes]>, T>
            where
                T: BytesSample<T> + 'a,
            {
                #[doc = "Create a new wrapper for an [Adapter] buffer of byte arrays, `[u8; " $bytes "]`,"]
                #[doc = "containing samples of type `" $typename "`."]
                pub fn new(
                    buf: &'a dyn Adapter<'a, [u8; $bytes]>,
                ) -> Self {
                    Self {
                        _phantom: core::marker::PhantomData,
                        buf,
                    }
                }
            }

            impl<'a, T> [< Convert $typename >]<&'a mut dyn AdapterMut<'a, [u8; $bytes]>, T>
            where
                T: BytesSample<T> + 'a,
            {
                #[doc = "Create a new wrapper for an mutable [AdapterMut] buffer of byte arrays, `[u8; " $bytes "]`,"]
                #[doc = "containing samples of type `" $typename "`."]
                pub fn new_mut(
                    buf: &'a mut dyn AdapterMut<'a, [u8; $bytes]>,
                ) -> Self {
                    Self {
                        _phantom: core::marker::PhantomData,
                        buf,
                    }
                }
            }

            impl<'a, T> Adapter<'a, T> for [< Convert $typename >]<&'a dyn Adapter<'a, [u8; $bytes]>, T>
            where
                T: BytesSample <T> + 'a,
            {
                unsafe fn read_sample_unchecked(&self, channel: usize, frame: usize) -> T {
                    T::$read_func(self.buf.read_sample_unchecked(channel, frame))
                }

                fn channels(&self) -> usize {
                    self.buf.channels()
                }

                fn frames(&self) -> usize {
                    self.buf.frames()
                }
            }

            impl<'a, T> Adapter<'a, T> for [< Convert $typename >]<&'a mut dyn AdapterMut<'a, [u8; $bytes]>, T>
            where
                T: BytesSample<T> + 'a,
            {
                unsafe fn read_sample_unchecked(&self, channel: usize, frame: usize) -> T {
                    T::$read_func(self.buf.read_sample_unchecked(channel, frame))
                }

                fn channels(&self) -> usize {
                    self.buf.channels()
                }

                fn frames(&self) -> usize {
                    self.buf.frames()
                }
            }

            impl<'a, T> AdapterMut<'a, T> for [< Convert $typename >]<&'a mut dyn AdapterMut<'a, [u8; $bytes]>, T>
            where
                T: BytesSample<T> + Clone + 'a,
            {
                unsafe fn write_sample_unchecked(&mut self, channel: usize, frame: usize, value: &T) -> bool {
                    let (value, clipped) = T::$write_func(value);
                    self.buf.write_sample_unchecked(channel, frame, &value);
                    clipped
                }
            }
        }
    };
}

byte_convert_structs!(2, S16LE);
byte_convert_structs!(2, S16BE);
byte_convert_structs!(3, S24LE3);
byte_convert_structs!(3, S24BE3);
byte_convert_structs!(4, S24LE4);
byte_convert_structs!(4, S24BE4);
byte_convert_structs!(4, S32LE);
byte_convert_structs!(4, S32BE);
byte_convert_structs!(4, F32LE);
byte_convert_structs!(4, F32BE);
byte_convert_structs!(8, F64LE);
byte_convert_structs!(8, F64BE);

byte_convert_traits!(from_s16_le, to_s16_le, 2, S16LE);
byte_convert_traits!(from_s16_be, to_s16_be, 2, S16BE);
byte_convert_traits!(from_s24_3_le, to_s24_3_le, 3, S24LE3);
byte_convert_traits!(from_s24_3_be, to_s24_3_be, 3, S24BE3);
byte_convert_traits!(from_s24_4_le, to_s24_4_le, 4, S24LE4);
byte_convert_traits!(from_s24_4_be, to_s24_4_be, 4, S24BE4);
byte_convert_traits!(from_s32_le, to_s32_le, 4, S32LE);
byte_convert_traits!(from_s32_be, to_s32_be, 4, S32BE);
byte_convert_traits!(from_f32_le, to_f32_le, 4, F32LE);
byte_convert_traits!(from_f32_be, to_f32_be, 4, F32BE);
byte_convert_traits!(from_f64_le, to_f64_le, 8, F64LE);
byte_convert_traits!(from_f64_be, to_f64_be, 8, F64BE);

macro_rules! int_convert_structs {
    ($type:expr, $typename:ident) => {
        paste::item! {
            #[doc = "A wrapper for an [Adapter] or [AdapterMut] buffer containing `" $type "` samples"]
            pub struct [< Convert $typename >]<U, V> {
                _phantom: core::marker::PhantomData<V>,
                buf: U,
            }
        }
    };
}

macro_rules! int_convert_traits {
    ($type:expr, $read_func:ident, $write_func:ident, $typename:ident) => {
        paste::item! {

            impl<'a, T> [< Convert $typename >]<&'a dyn Adapter<'a, $type>, T>
            where
                T: NumericSample<T> + 'a,
            {
                #[doc = "Create a new wrapper for a buffer implementing the [Adapter] trait,"]
                #[doc = "containing samples of type `" $type "`."]
                pub fn new(
                    buf: &'a dyn Adapter<'a, $type>,
                ) -> Self {
                    Self {
                        _phantom: core::marker::PhantomData,
                        buf,
                    }
                }
            }

            impl<'a, T> [< Convert $typename >]<&'a mut dyn AdapterMut<'a, $type>, T>
            where
                T: NumericSample<T> + 'a,
            {
                #[doc = "Create a new wrapper for a mutable buffer implementing the [AdapterMut] trait,"]
                #[doc = "containing samples of type `" $type "`."]
                pub fn new_mut(
                    buf: &'a mut dyn AdapterMut<'a, $type>,
                ) -> Self {
                    Self {
                        _phantom: core::marker::PhantomData,
                        buf,
                    }
                }
            }

            impl<'a, T> Adapter<'a, T> for [< Convert $typename >]<&'a dyn Adapter<'a, $type>, T>
            where
                T: NumericSample <T> + 'a,
            {
                unsafe fn read_sample_unchecked(&self, channel: usize, frame: usize) -> T {
                    T::$read_func(self.buf.read_sample_unchecked(channel, frame))
                }

                fn channels(&self) -> usize {
                    self.buf.channels()
                }

                fn frames(&self) -> usize {
                    self.buf.frames()
                }
            }

            impl<'a, T> Adapter<'a, T> for [< Convert $typename >]<&'a mut dyn AdapterMut<'a, $type>, T>
            where
                T: NumericSample<T> + 'a,
            {
                unsafe fn read_sample_unchecked(&self, channel: usize, frame: usize) -> T {
                    T::$read_func(self.buf.read_sample_unchecked(channel, frame))
                }

                fn channels(&self) -> usize {
                    self.buf.channels()
                }

                fn frames(&self) -> usize {
                    self.buf.frames()
                }
            }

            impl<'a, T> AdapterMut<'a, T> for [< Convert $typename >]<&'a mut dyn AdapterMut<'a, $type>, T>
            where
                T: NumericSample<T> + Clone + 'a,
            {
                unsafe fn write_sample_unchecked(&mut self, channel: usize, frame: usize, value: &T) -> bool {
                    let (value, clipped) = T::$write_func(value);
                    self.buf.write_sample_unchecked(channel, frame, &value);
                    clipped
                }
            }
        }
    };
}

int_convert_structs!(i8, I8);
int_convert_structs!(i16, I16);
int_convert_structs!(i32, I32);
int_convert_structs!(f32, F32);
int_convert_structs!(f64, F64);

int_convert_traits!(i8, from_i8, to_i8, I8);
int_convert_traits!(i16, from_i16, to_i16, I16);
int_convert_traits!(i32, from_i32, to_i32, I32);
int_convert_traits!(f32, from_f32, to_f32, F32);
int_convert_traits!(f64, from_f64, to_f64, F64);

//   _____         _
//  |_   _|__  ___| |_ ___
//    | |/ _ \/ __| __/ __|
//    | |  __/\__ \ |_\__ \
//    |_|\___||___/\__|___/

#[cfg(test)]
mod tests {
    use super::*;
    use crate::direct::InterleavedSlice;
    use crate::Adapter;

    #[test]
    fn read_i16_bytes() {
        let data: [[u8; 2]; 6] = [[0, 0], [0, 128], [0, 64], [0, 192], [0, 32], [0, 224]];
        let buffer: InterleavedSlice<&[[u8; 2]]> = InterleavedSlice::new(&data, 2, 3).unwrap();
        let converter: ConvertS16LE<&dyn Adapter<[u8; 2]>, f32> =
            ConvertS16LE::new(&buffer as &dyn Adapter<[u8; 2]>);
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
        let converter: ConvertI16<&dyn Adapter<i16>, f32> =
            ConvertI16::new(&buffer as &dyn Adapter<i16>);
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
        let mut converter: ConvertS16LE<&mut dyn AdapterMut<[u8; 2]>, f32> =
            ConvertS16LE::new_mut(&mut buffer as &mut dyn AdapterMut<[u8; 2]>);
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
        let mut converter: ConvertI16<&mut dyn AdapterMut<i16>, f32> =
            ConvertI16::new_mut(&mut buffer as &mut dyn AdapterMut<i16>);
        converter.write_sample(0, 0, &0.0).unwrap();
        converter.write_sample(1, 0, &-1.0).unwrap();
        converter.write_sample(0, 1, &0.5).unwrap();
        converter.write_sample(1, 1, &-0.5).unwrap();
        converter.write_sample(0, 2, &0.25).unwrap();
        converter.write_sample(1, 2, &-0.25).unwrap();
        assert_eq!(data, expected);
    }
}
