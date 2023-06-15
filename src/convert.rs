use crate::{
    check_slice_length, implement_size_getters, BufferSizeError,
};
use rawsample::Sample;

// -------------------- The main buffer trait --------------------

/// A trait for providing immutable access to samples in a buffer.
pub trait ConvertingAudioBuffer<'a, T: 'a> {
    /// Get an immutable reference to the sample at
    /// a given combination of frame and channel.
    ///
    /// # Safety
    ///
    /// This method performs no bounds checking.
    /// Calling it with an out-of-bound value for frame or channel
    /// results in undefined behavior,
    /// for example returning an invalid value or panicking.
    unsafe fn read_unchecked(&self, channel: usize, frame: usize) -> T;

    /// Get an immutable reference to the sample at
    /// a given combination of frame and channel.
    /// Returns `None` if the frame or channel is
    /// out of bounds of the `AudioBuffer`.
    fn read(&self, channel: usize, frame: usize) -> Option<T> {
        if channel >= self.channels() || frame >= self.frames() {
            return None;
        }
        Some(unsafe { self.read_unchecked(channel, frame) })
    }

    /// Get the number of channels stored in this `AudioBuffer`.
    fn channels(&self) -> usize;

    /// Get the number of frames stored in this `AudioBuffer`.
    fn frames(&self) -> usize;

    /// Write values from channel of the `AudioBuffer` to a slice.
    /// The `start` argument is the offset into the `AudioBuffer` channel
    /// where the first value will be read from.
    /// If the slice is longer than the available number of values in the `AudioBuffer` channel,
    /// then only the available number of samples will be written.
    ///
    /// Returns the number of values written.
    /// If an invalid channel number is given,
    /// or if `start` is larger than the length of the channel,
    /// no samples will be written and zero is returned.
    fn write_from_channel_to_slice(&self, channel: usize, start: usize, slice: &mut [T]) -> usize {
        if channel >= self.channels() || start >= self.frames() {
            return 0;
        }
        let frames_to_write = if (self.frames() - start) < slice.len() {
            self.frames() - start
        } else {
            slice.len()
        };
        for (n, item) in slice.iter_mut().enumerate().take(frames_to_write) {
            unsafe { *item = self.read_unchecked(channel, start + n) };
        }
        frames_to_write
    }

    /// Write values from a frame of the `AudioBuffer` to a slice.
    /// The `start` argument is the offset into the `AudioBuffer` frame
    /// where the first value will be read from.
    /// If the slice is longer than the available number of values in the `AudioBuffer` frame,
    /// then only the available number of samples will be written.
    ///
    /// Returns the number of values written.
    /// If an invalid frame number is given,
    /// or if `start` is larger than the length of the frame,
    /// no samples will be written and zero is returned.
    fn write_from_frame_to_slice(&self, frame: usize, start: usize, slice: &mut [T]) -> usize {
        if frame >= self.frames() || start >= self.channels() {
            return 0;
        }
        let channels_to_write = if (self.channels() - start) < slice.len() {
            self.channels() - start
        } else {
            slice.len()
        };
        for (n, item) in slice.iter_mut().enumerate().take(channels_to_write) {
            unsafe { *item = self.read_unchecked(start + n, frame) };
        }
        channels_to_write
    }
}

/// A trait for providing mutable access to samples in a buffer.
pub trait ConvertingAudioBufferMut<'a, T>: ConvertingAudioBuffer<'a, T>
where
    T: Clone + 'a,
{
    /// Get a mutable reference to the sample at
    /// a given combination of frame and channel.
    ///
    /// # Safety
    ///
    /// This method performs no bounds checking.
    /// Calling it with an out-of-bound value for frame or channel
    /// results in undefined behavior,
    /// for example returning an invalid value or panicking.
    unsafe fn write_unchecked(&mut self, channel: usize, frame: usize, value: &T) -> bool;

    /// Get a mutable reference to the sample at
    /// a given combination of frame and channel.
    /// Returns `None` if the frame or channel is
    /// out of bounds of the `AudioBuffer`.
    fn write(&mut self, channel: usize, frame: usize, value: &T) -> Option<bool> {
        if channel >= self.channels() || frame >= self.frames() {
            return None;
        }
        Some(unsafe { self.write_unchecked(channel, frame, value) })
    }

    /// Read values from a slice into a channel of the `AudioBuffer`.
    /// The `start` argument is the offset into the `AudioBuffer` channel
    /// where the first value will be written.
    /// If the slice is longer than the available space in the `AudioBuffer` channel,
    /// then only the number of samples that fit will be read.
    ///
    /// Returns the number of values read.
    /// If an invalid channel number is given,
    /// or if `start` is larger than the length of the channel,
    /// no samples will be read and zero is returned.
    fn read_into_channel_from_slice(&mut self, channel: usize, start: usize, slice: &[T]) -> usize {
        if channel >= self.channels() || start >= self.frames() {
            return 0;
        }
        let frames_to_read = if (self.frames() - start) < slice.len() {
            self.frames() - start
        } else {
            slice.len()
        };
        for (n, item) in slice.iter().enumerate().take(frames_to_read) {
            unsafe { self.write_unchecked(channel, start + n, item) };
        }
        frames_to_read
    }

    /// Read values from a slice into a frame of the `AudioBuffer`.
    /// The `start` argument is the offset into the `AudioBuffer` frame
    /// where the first value will be written.
    /// If the slice is longer than the available space in the `AudioBuffer` frame,
    /// then only the number of samples that fit will be read.
    ///
    /// Returns the number of values read.
    /// If an invalid frame number is given,
    /// or if `start` is larger than the length of the frame,
    /// no samples will be read and zero is returned.
    fn read_into_frame_from_slice(&mut self, frame: usize, start: usize, slice: &[T]) -> usize {
        if frame >= self.frames() || start >= self.channels() {
            return 0;
        }
        let channels_to_read = if (self.channels() - start) < slice.len() {
            self.channels() - start
        } else {
            slice.len()
        };
        for (n, item) in slice.iter().enumerate().take(channels_to_read) {
            unsafe { self.write_unchecked(start + n, frame, item) };
        }
        channels_to_read
    }
}

macro_rules! implement_converting_buffer {
    ($type:expr, $read_func:ident, $write_func:ident, $bytes:expr, $name:ident) => {
        pub struct $name<U, V> {
            _phantom: core::marker::PhantomData<V>,
            buf: U,
            frames: usize,
            channels: usize,
            bytes_per_sample: usize,
        }

        impl<U, V> $name<U, V> {
            fn calc_index(&self, channel: usize, frame: usize) -> usize {
                self.bytes_per_sample * (frame * self.channels + channel)
            }
        }

        impl<'a, T> $name<&'a [u8], T>
        where
            T: 'a,
        {
            /// Create a new `InterleavedWrapper` to wrap a slice.
            /// The slice length must be at least `frames*channels`.
            /// It is allowed to be longer than needed,
            /// but these extra values cannot
            /// be accessed via the `AudioBuffer` trait methods.
            pub fn new(
                buf: &'a [u8],
                channels: usize,
                frames: usize,
            ) -> Result<Self, BufferSizeError> {
                check_slice_length!(channels, frames, buf.len(), $bytes, Slice);
                Ok(Self {
                    _phantom: core::marker::PhantomData,
                    buf,
                    frames,
                    channels,
                    bytes_per_sample: $bytes,
                })
            }
        }

        impl<'a, T> $name<&'a mut [u8], T>
        where
            T: 'a,
        {
            /// Create a new `InterleavedWrapper` to wrap a slice.
            /// The slice length must be at least `frames*channels`.
            /// It is allowed to be longer than needed,
            /// but these extra values cannot
            /// be accessed via the `AudioBuffer` trait methods.
            pub fn new_mut(
                buf: &'a mut [u8],
                channels: usize,
                frames: usize,
            ) -> Result<Self, BufferSizeError> {
                check_slice_length!(channels, frames, buf.len(), $bytes, Slice);
                Ok(Self {
                    _phantom: core::marker::PhantomData,
                    buf,
                    frames,
                    channels,
                    bytes_per_sample: $bytes,
                })
            }
        }

        impl<'a, T> ConvertingAudioBuffer<'a, T> for $name<&'a [u8], T>
        where
            T: Sample<T> + 'a,
        {
            unsafe fn read_unchecked(&self, channel: usize, frame: usize) -> T {
                let index = self.calc_index(channel, frame);
                T::$read_func(
                    self.buf[index..index + self.bytes_per_sample]
                        .try_into()
                        .unwrap(),
                )
            }

            implement_size_getters!();
        }

        impl<'a, T> ConvertingAudioBuffer<'a, T> for $name<&'a mut [u8], T>
        where
            T: Sample<T> + Clone + 'a,
        {
            unsafe fn read_unchecked(&self, channel: usize, frame: usize) -> T {
                let index = self.calc_index(channel, frame);
                T::$read_func(
                    self.buf[index..index + self.bytes_per_sample]
                        .try_into()
                        .unwrap(),
                )
            }

            implement_size_getters!();
        }

        impl<'a, T> ConvertingAudioBufferMut<'a, T> for $name<&'a mut [u8], T>
        where
            T: Sample<T> + Clone + 'a,
        {
            unsafe fn write_unchecked(&mut self, channel: usize, frame: usize, value: &T) -> bool {
                let index = self.calc_index(channel, frame);
                let (value, clipped) = T::$write_func(value);
                self.buf[index..index + self.bytes_per_sample].clone_from_slice(&value);
                clipped
            }
        }
    };
}

implement_converting_buffer!(i16, from_s16_le, to_s16_le, 2, ConvertingInterleavedI16LE);
implement_converting_buffer!(i16, from_s16_be, to_s16_be, 2, ConvertingInterleavedI16BE);
implement_converting_buffer!(
    i16,
    from_s24_3_le,
    to_s24_3_le,
    3,
    ConvertingInterleavedI243LE
);
implement_converting_buffer!(
    i16,
    from_s24_3_be,
    to_s24_3_be,
    3,
    ConvertingInterleavedI243BE
);
implement_converting_buffer!(
    i16,
    from_s24_4_le,
    to_s24_4_le,
    4,
    ConvertingInterleavedI244LE
);
implement_converting_buffer!(
    i16,
    from_s24_4_be,
    to_s24_4_be,
    4,
    ConvertingInterleavedI244BE
);
implement_converting_buffer!(i32, from_s32_le, to_s32_le, 4, ConvertingInterleavedI32LE);
implement_converting_buffer!(i32, from_s32_be, to_s32_be, 4, ConvertingInterleavedI32BE);
implement_converting_buffer!(f32, from_f32_le, to_f32_le, 4, ConvertingInterleavedF32LE);
implement_converting_buffer!(f32, from_f32_be, to_f32_be, 4, ConvertingInterleavedF32BE);
implement_converting_buffer!(f64, from_f64_le, to_f64_le, 8, ConvertingInterleavedF64LE);
implement_converting_buffer!(f64, from_f64_be, to_f64_be, 8, ConvertingInterleavedF64BE);

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
        let data: Vec<u8> = vec![
            0, 0, 0, 0, 0, 0, 0, 128, 0, 0, 0, 64, 0, 0, 0, 192, 0, 0, 0, 32, 0, 0, 0, 224,
        ];
        let buffer: ConvertingInterleavedI32LE<&[u8], f32> =
            ConvertingInterleavedI32LE::new(&data, 2, 3).unwrap();
        assert_eq!(buffer.read(0, 0).unwrap(), 0.0);
        assert_eq!(buffer.read(1, 0).unwrap(), -1.0);
        assert_eq!(buffer.read(0, 1).unwrap(), 0.5);
        assert_eq!(buffer.read(1, 1).unwrap(), -0.5);
        assert_eq!(buffer.read(0, 2).unwrap(), 0.25);
        assert_eq!(buffer.read(1, 2).unwrap(), -0.25);
    }

    #[test]
    fn read_i16() {
        let data: Vec<u8> = vec![0, 0, 0, 128, 0, 64, 0, 192, 0, 32, 0, 224];
        let buffer: ConvertingInterleavedI16LE<&[u8], f32> =
            ConvertingInterleavedI16LE::new(&data, 2, 3).unwrap();
        assert_eq!(buffer.read(0, 0).unwrap(), 0.0);
        assert_eq!(buffer.read(1, 0).unwrap(), -1.0);
        assert_eq!(buffer.read(0, 1).unwrap(), 0.5);
        assert_eq!(buffer.read(1, 1).unwrap(), -0.5);
        assert_eq!(buffer.read(0, 2).unwrap(), 0.25);
        assert_eq!(buffer.read(1, 2).unwrap(), -0.25);
    }

    #[test]
    fn write_i32() {
        let expected: Vec<u8> = vec![
            0, 0, 0, 0, 0, 0, 0, 128, 0, 0, 0, 64, 0, 0, 0, 192, 0, 0, 0, 32, 0, 0, 0, 224,
        ];
        let mut data = vec![0; 24];
        let mut buffer: ConvertingInterleavedI32LE<&mut [u8], f32> =
            ConvertingInterleavedI32LE::new_mut(&mut data, 2, 3).unwrap();

        buffer.write(0, 0, &0.0).unwrap();
        buffer.write(1, 0, &-1.0).unwrap();
        buffer.write(0, 1, &0.5).unwrap();
        buffer.write(1, 1, &-0.5).unwrap();
        buffer.write(0, 2, &0.25).unwrap();
        buffer.write(1, 2, &-0.25).unwrap();
        assert_eq!(data, expected);
    }

    #[test]
    fn write_i16() {
        let expected: Vec<u8> = vec![0, 0, 0, 128, 0, 64, 0, 192, 0, 32, 0, 224];
        let mut data = vec![0; 12];
        let mut buffer: ConvertingInterleavedI16LE<&mut [u8], f32> =
            ConvertingInterleavedI16LE::new_mut(&mut data, 2, 3).unwrap();

        buffer.write(0, 0, &0.0).unwrap();
        buffer.write(1, 0, &-1.0).unwrap();
        buffer.write(0, 1, &0.5).unwrap();
        buffer.write(1, 1, &-0.5).unwrap();
        buffer.write(0, 2, &0.25).unwrap();
        buffer.write(1, 2, &-0.25).unwrap();
        assert_eq!(data, expected);
    }
}
