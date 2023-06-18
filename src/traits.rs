//! # audiobuffer_traits
//! 
//! A set of traits for making it easier to work with buffers of audio data.
//! 
//! Audio data can be stored in many different ways,
//! where both the layout of the data, and the numerical representation can vary.
//! This crate aims at providing traits that make it easy to write applications
//! that can handle any data type in any data layout.
//! 
//! 
//! ## Abstracting the data layout
//! This module provides several "layers" of traits that add more functionality.
//! The most basic traits are [Converter] and [ConverterMut]. These provide basic reading and writing.
//! 
//! The next level is the [AudioBuffer] and [AudioBufferMut] traits, that add immutable and mutable
//! borrowing, and iterators.
//! 
//! The last level is [AudioBufferStats] that is used to calculate some properties of the audio data.
//! This is implemented for every structure implementing [AudioBuffer] for a numeric type.
//! 
//! By accessing the audio data via the trait methods instead
//! of indexing the data structure directly,
//! an application or library becomes independant of the data layout.
//! 
//! ## Supporting new data structures
//! The required trait methods are simple, to make is easy to implement them for
//! data structures not covered by the existing wrappers in [audioboiler_buffer].
//! 
//! There are default implementations for most methods.
//! These may be overriden if the wrapped data structure provides a more efficient way
//! of performing the operation.
//! For example, the default implementation of `write_from_channel_to_slice()`
//! simply loops over the elements to copy.
//! But when the underlying data structure is a sequential slice, then this
//! can be implemented more efficiently by using [slice::clone_from_slice()].
//! 
//! 
//! ## License: MIT
//!


use crate::iterators::{Frames, FramesMut, Channels, ChannelsMut, ChannelSamples, ChannelSamplesMut, FrameSamples, FrameSamplesMut};




#[macro_export]
macro_rules! implement_iterators {
    () => {
        fn iter_channel(&self, channel: usize) -> Option<ChannelSamples<'a, '_, T>> {
            ChannelSamples::new(self, channel)
        }

        fn iter_channels(&self) -> Channels<'a, '_, T> {
            Channels::new(self)
        }

        fn iter_frame(&self, frame: usize) -> Option<FrameSamples<'a, '_, T>> {
            FrameSamples::new(self, frame)
        }

        fn iter_frames(&self) -> Frames<'a, '_, T> {
            Frames::new(self)
        }
    };
}

#[macro_export]
macro_rules! implement_iterators_mut {
    () => {
        fn iter_channel_mut(&mut self, channel: usize) -> Option<ChannelSamplesMut<'a, '_, T>> {
            ChannelSamplesMut::new(self, channel)
        }

        fn iter_channels_mut(&mut self) -> ChannelsMut<'a, '_, T> {
            ChannelsMut::new(self)
        }

        fn iter_frame_mut(&mut self, frame: usize) -> Option<FrameSamplesMut<'a, '_, T>> {
            FrameSamplesMut::new(self, frame)
        }

        fn iter_frames_mut(&mut self) -> FramesMut<'a, '_, T> {
            FramesMut::new(self)
        }
    };
}


// -------------------- The main buffer trait --------------------

/// A trait for providing immutable access to samples in a buffer.
/// Samples are converted from the raw format on the fly.
pub trait Converter<'a, T: 'a> {
    /// Read and convert the sample at
    /// a given combination of frame and channel.
    ///
    /// # Safety
    ///
    /// This method performs no bounds checking.
    /// Calling it with an out-of-bound value for frame or channel
    /// results in undefined behavior,
    /// for example returning an invalid value or panicking.
    unsafe fn read_unchecked(&self, channel: usize, frame: usize) -> T;

    /// Read and convert the sample at
    /// a given combination of frame and channel.
    /// Returns `None` if the frame or channel is
    /// out of bounds of the `Converter`.
    fn read(&self, channel: usize, frame: usize) -> Option<T> {
        if channel >= self.channels() || frame >= self.frames() {
            return None;
        }
        Some(unsafe { self.read_unchecked(channel, frame) })
    }

    /// Get the number of channels stored in this `Converter`.
    fn channels(&self) -> usize;

    /// Get the number of frames stored in this `Converter`.
    fn frames(&self) -> usize;

    /// Convert and write values from a channel of the `Converter` to a slice.
    /// The `start` argument is the offset into the `Converter` channel
    /// where the first value will be read from.
    /// If the slice is longer than the available number of values in the `Converter` channel,
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

    /// Convert and write values from a frame of the `Converter` to a slice.
    /// The `start` argument is the offset into the `Converter` frame
    /// where the first value will be read from.
    /// If the slice is longer than the available number of values in the `Converter` frame,
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
/// Samples are converted to the raw format on the fly.
pub trait ConverterMut<'a, T>: Converter<'a, T>
where
    T: Clone + 'a,
{
    /// Convert and write a sample to the
    /// given combination of frame and channel.
    ///
    /// # Safety
    ///
    /// This method performs no bounds checking.
    /// Calling it with an out-of-bound value for frame or channel
    /// results in undefined behavior,
    /// for example returning an invalid value or panicking.
    unsafe fn write_unchecked(&mut self, channel: usize, frame: usize, value: &T) -> bool;

    /// Convert and write a sample to the
    /// given combination of frame and channel.
    /// Returns `None` if the frame or channel is
    /// out of bounds of the `Converter`.
    fn write(&mut self, channel: usize, frame: usize, value: &T) -> Option<bool> {
        if channel >= self.channels() || frame >= self.frames() {
            return None;
        }
        Some(unsafe { self.write_unchecked(channel, frame, value) })
    }

    /// Write values from a slice into a channel of the `Converter`.
    /// The `start` argument is the offset into the `Converter` channel
    /// where the first value will be written.
    /// If the slice is longer than the available space in the `Converter` channel,
    /// then only the number of samples that fit will be read.
    ///
    /// Returns a tuple of two numbers.
    /// The first is the number of values written,
    /// and the second is the number of values that were clipped during conversion.
    /// If an invalid channel number is given,
    /// or if `start` is larger than the length of the channel,
    /// no samples will be read and (0, 0) is returned.
    fn write_from_slice_to_channel(
        &mut self,
        channel: usize,
        start: usize,
        slice: &[T],
    ) -> (usize, usize) {
        if channel >= self.channels() || start >= self.frames() {
            return (0, 0);
        }
        let frames_to_read = if (self.frames() - start) < slice.len() {
            self.frames() - start
        } else {
            slice.len()
        };
        let mut nbr_clipped = 0;
        for (n, item) in slice.iter().enumerate().take(frames_to_read) {
            unsafe { nbr_clipped += self.write_unchecked(channel, start + n, item) as usize };
        }
        (frames_to_read, nbr_clipped)
    }

    /// Write values from a slice into a frame of the `Converter`.
    /// The `start` argument is the offset into the `Converter` frame
    /// where the first value will be written.
    /// If the slice is longer than the available space in the `Converter` frame,
    /// then only the number of samples that fit will be read.
    ///
    /// Returns a tuple of two numbers.
    /// The first is the number of values written,
    /// and the second is the number of values that were clipped during conversion.
    /// If an invalid frame number is given,
    /// or if `start` is larger than the length of the frame,
    /// no samples will be read and (0, 0) is returned.
    fn write_from_slice_to_frame(
        &mut self,
        frame: usize,
        start: usize,
        slice: &[T],
    ) -> (usize, usize) {
        if frame >= self.frames() || start >= self.channels() {
            return (0, 0);
        }
        let channels_to_read = if (self.channels() - start) < slice.len() {
            self.channels() - start
        } else {
            slice.len()
        };
        let mut nbr_clipped = 0;
        for (n, item) in slice.iter().enumerate().take(channels_to_read) {
            unsafe { nbr_clipped += self.write_unchecked(start + n, frame, item) as usize };
        }
        (channels_to_read, nbr_clipped)
    }
}


// -------------------- The main buffer trait --------------------

/// A trait for providing immutable access to samples in a buffer.
pub trait AudioBuffer<'a, T: Clone + 'a> {
    /// Get an immutable reference to the sample at
    /// a given combination of frame and channel.
    ///
    /// # Safety
    ///
    /// This method performs no bounds checking.
    /// Calling it with an out-of-bound value for frame or channel
    /// results in undefined behavior,
    /// for example returning an invalid value or panicking.
    unsafe fn get_unchecked(&self, channel: usize, frame: usize) -> &T;

    /// Get an immutable reference to the sample at
    /// a given combination of frame and channel.
    /// Returns `None` if the frame or channel is
    /// out of bounds of the `AudioBuffer`.
    fn get(&self, channel: usize, frame: usize) -> Option<&T> {
        if channel >= self.channels() || frame >= self.frames() {
            return None;
        }
        Some(unsafe { self.get_unchecked(channel, frame) })
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
            unsafe { *item = self.get_unchecked(channel, start + n).clone() };
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
            unsafe { *item = self.get_unchecked(start + n, frame).clone() };
        }
        channels_to_write
    }

    /// Returns an iterator that yields immutable references to the samples of a channel.
    fn iter_channel(&self, channel: usize) -> Option<ChannelSamples<'a, '_, T>>;

    /// Returns an iterator that runs over the available channels of the `AudioBuffer`.
    /// Each element is an iterator that yields immutable references to the samples of the channel.
    fn iter_channels(&self) -> Channels<'a, '_, T>;

    /// Returns an iterator that yields immutable references to the samples of a frame.
    fn iter_frame(&self, frame: usize) -> Option<FrameSamples<'a, '_, T>>;

    /// Returns an iterator that runs over the available frames of the `AudioBuffer`.
    /// Each element is an iterator that yields immutable references to the samples of the frame.
    fn iter_frames(&self) -> Frames<'a, '_, T>;
}

/// A trait for providing mutable access to samples in a buffer.
pub trait AudioBufferMut<'a, T: Clone + 'a>: AudioBuffer<'a, T> {
    /// Get a mutable reference to the sample at
    /// a given combination of frame and channel.
    ///
    /// # Safety
    ///
    /// This method performs no bounds checking.
    /// Calling it with an out-of-bound value for frame or channel
    /// results in undefined behavior,
    /// for example returning an invalid value or panicking.
    unsafe fn get_unchecked_mut(&mut self, channel: usize, frame: usize) -> &mut T;

    /// Get a mutable reference to the sample at
    /// a given combination of frame and channel.
    /// Returns `None` if the frame or channel is
    /// out of bounds of the `AudioBuffer`.
    fn get_mut(&mut self, channel: usize, frame: usize) -> Option<&mut T> {
        if channel >= self.channels() || frame >= self.frames() {
            return None;
        }
        Some(unsafe { self.get_unchecked_mut(channel, frame) })
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
            unsafe { *self.get_unchecked_mut(channel, start + n) = item.clone() };
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
            unsafe { *self.get_unchecked_mut(start + n, frame) = item.clone() };
        }
        channels_to_read
    }

    /// Returns an iterator that yields mutable references to the samples of a channel.
    fn iter_channel_mut(&mut self, channel: usize) -> Option<ChannelSamplesMut<'a, '_, T>>;

    /// Returns an iterator that runs over the available channels of the `AudioBuffer`.
    /// Each element is an iterator that yields mutable references to the samples of the channel.
    fn iter_channels_mut(&mut self) -> ChannelsMut<'a, '_, T>;

    /// Returns an iterator that yields mutable references to the samples of a frame.
    fn iter_frame_mut(&mut self, frame: usize) -> Option<FrameSamplesMut<'a, '_, T>>;

    /// Returns an iterator that runs over the available frames of the `AudioBuffer`.
    /// Each element is an iterator that yields mutable references to the samples of the frame.
    fn iter_frames_mut(&mut self) -> FramesMut<'a, '_, T>;
}


