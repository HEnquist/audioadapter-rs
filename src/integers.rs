//! # Converting wrappers for integers
//! This module provides wrappers for slices of integers.
//! The wrapper enables reading and writing samples from/to the slice with
//! on-the-fly format conversion between integer and float.
//!
//! The wrappers implement the traits [crate::Indirect] and [crate::IndirectMut],
//! that provide simple methods for accessing the audio samples of a buffer.
//!
//! ### Data order
//! There are two wrappers availabe for each sample format,
//! one for interleaved and one for sequential data.
//!
//! ### Example
//! Wrap a Vec of 16-bit integer samples as an interleaved buffer
//! and print all the values.
//! ```
//! use audioadapter::integers::InterleavedI16;
//! use audioadapter::Indirect;
//!
//! // make a vector with some data.
//! // 2 channels * 3 frames => 6 samples
//! let data: Vec<i16> = vec![1, 2, 3, 4, 5, 6];
//!
//! // wrap the data
//! let buffer: InterleavedI16<&[i16], f32> = InterleavedI16::new(&data, 2, 3).unwrap();
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

use crate::SizeError;
use crate::{check_slice_length, implement_size_getters};
use crate::{Indirect, IndirectMut};
use rawsample::IntegerSample;

macro_rules! create_structs {
    ($type:expr, $read_func:ident, $write_func:ident, $typename:ident) => {
        paste::item! {
            #[doc = "A wrapper for a slice containing interleaved `" $type "` samples."]
            pub struct [< Interleaved $typename >]<U, V> {
                _phantom: core::marker::PhantomData<V>,
                buf: U,
                frames: usize,
                channels: usize,
            }

            #[doc = "A wrapper for a slice containing interleaved `" $type "` samples."]
            pub struct [< Sequential $typename >]<U, V> {
                _phantom: core::marker::PhantomData<V>,
                buf: U,
                frames: usize,
                channels: usize,
            }

            impl<U, V> [< Interleaved $typename >]<U, V> {
                fn calc_index(&self, channel: usize, frame: usize) -> usize {
                    frame * self.channels + channel
                }
            }

            impl<U, V> [< Sequential $typename >]<U, V> {
                fn calc_index(&self, channel: usize, frame: usize) -> usize {
                    frame + channel * self.frames
                }
            }
        }
    };
}

macro_rules! impl_traits {
    ($type:expr, $read_func:ident, $write_func:ident, $typename:ident, $order:ident) => {
        paste::item! {


            impl<'a, T> [< $order $typename >]<&'a [$type], T>
            where
                T: 'a,
            {
                #[doc = "Create a new wrapper for an immutable slice"]
                #[doc = "of `" $type "` samples"]
                #[doc = "stored in _" $order:lower "_ order."]
                #[doc = "The slice length must be at least `frames*channels`."]
                #[doc = "It is allowed to be longer than needed,"]
                #[doc = "but these extra values cannot"]
                #[doc = "be accessed via the `Indirect` trait methods."]
                pub fn new(
                    buf: &'a [$type],
                    channels: usize,
                    frames: usize,
                ) -> Result<Self, SizeError> {
                    check_slice_length!(channels, frames, buf.len());
                    Ok(Self {
                        _phantom: core::marker::PhantomData,
                        buf,
                        frames,
                        channels,
                    })
                }
            }

            impl<'a, T> [< $order $typename >]<&'a mut [$type], T>
            where
                T: 'a,
            {
                #[doc = "Create a new wrapper for a mutable slice"]
                #[doc = "of `" $type "` samples"]
                #[doc = "stored in _" $order:lower "_ order."]
                #[doc = "The slice length must be at least `frames*channels`."]
                #[doc = "It is allowed to be longer than needed,"]
                #[doc = "but these extra values cannot"]
                #[doc = "be accessed via the `Indirect` or `IndirectMut` trait methods."]
                pub fn new_mut(
                    buf: &'a mut [$type],
                    channels: usize,
                    frames: usize,
                ) -> Result<Self, SizeError> {
                    check_slice_length!(channels, frames, buf.len());
                    Ok(Self {
                        _phantom: core::marker::PhantomData,
                        buf,
                        frames,
                        channels,
                    })
                }
            }

            impl<'a, T> Indirect<'a, T> for [< $order $typename >]<&'a [$type], T>
            where
                T: IntegerSample<T> + 'a,
            {
                unsafe fn read_sample_unchecked(&self, channel: usize, frame: usize) -> T {
                    let index = self.calc_index(channel, frame);
                    T::$read_func(
                        self.buf[index]
                    )
                }

                implement_size_getters!();
            }

            impl<'a, T> Indirect<'a, T> for [< $order $typename >]<&'a mut [$type], T>
            where
                T: IntegerSample<T> + Clone + 'a,
            {
                unsafe fn read_sample_unchecked(&self, channel: usize, frame: usize) -> T {
                    let index = self.calc_index(channel, frame);
                    T::$read_func(
                        self.buf[index]
                    )
                }

                implement_size_getters!();
            }

            impl<'a, T> IndirectMut<'a, T> for [< $order $typename >]<&'a mut [$type], T>
            where
                T: IntegerSample<T> + Clone + 'a,
            {
                unsafe fn write_sample_unchecked(&mut self, channel: usize, frame: usize, value: &T) -> bool {
                    let index = self.calc_index(channel, frame);
                    let (value, clipped) = T::$write_func(value);
                    self.buf[index] = value;
                    clipped
                }
            }
        }
    };
}

create_structs!(i16, from_i16, to_i16, I16);
create_structs!(i32, from_i32, to_i32, I32);

impl_traits!(i16, from_i16, to_i16, I16, Interleaved);
impl_traits!(i32, from_i32, to_i32, I32, Interleaved);
impl_traits!(i16, from_i16, to_i16, I16, Sequential);
impl_traits!(i32, from_i32, to_i32, I32, Sequential);

//   _____         _
//  |_   _|__  ___| |_ ___
//    | |/ _ \/ __| __/ __|
//    | |  __/\__ \ |_\__ \
//    |_|\___||___/\__|___/

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_i32() {
        let data: [i32; 6] = [0, -2 << 30, 2 << 29, -2 << 29, 2 << 28, -2 << 28];
        let buffer: InterleavedI32<&[i32], f32> = InterleavedI32::new(&data, 2, 3).unwrap();
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
        let buffer: InterleavedI16<&[i16], f32> = InterleavedI16::new(&data, 2, 3).unwrap();
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
        let mut buffer: InterleavedI32<&mut [i32], f32> =
            InterleavedI32::new_mut(&mut data, 2, 3).unwrap();

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
        let mut buffer: InterleavedI16<&mut [i16], f32> =
            InterleavedI16::new_mut(&mut data, 2, 3).unwrap();

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
        let mut buffer: InterleavedI32<&mut [i32], f32> =
            InterleavedI32::new_mut(&mut data, 2, 3).unwrap();

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
        let buffer: InterleavedI32<&[i32], f32> = InterleavedI32::new(&data, 2, 3).unwrap();

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
        is_send::<InterleavedI32<&[i32], f32>>();
        is_sync::<InterleavedI32<&[i32], f32>>();
    }
}
