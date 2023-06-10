//! # AudioBuffer
//!
//! A simple library for making it easier to work with buffers of audio data.
//!
//! Audio data can be stored in many different ways,
//! where both the layout o the data, and the numerical representation can vary.
//! This crate aims at helping with the differences in layout.
//!
//! ## Background
//! Libraries and appications that process audio usually use
//! a single layout for the audio data internally.
//! If a project combines libraries that store their audio data differently,
//! any data passed between them must be converted
//! by copying the data from a buffer using one layout
//! to another buffer using the other layout.
//!
//! ## Abstracting the data layout
//! This crate provedes a trait [AudioBuffer] that provides simple methods
//! for accessing the audio samples of a buffer.
//! It also provides wrappers for a number of common data structures
//! used for storing audio data.
//!
//! By accessing the audio data via the trait methods instead
//! of indexing the data structure directly,
//! an application or library becomes independant of the data layout.
//!
//! ## Supporting new data structures
//! The required trait methods are simple, to make is easy to implement them for
//! data structures not covered by the built-in wrappers.
//!
//! There are default implementations for the functions that read and write slices.
//! These loop over the elements to read or write and clone element by element.
//! These may be overriden if the wrapped data structure provides a more efficient way
//! of cloning the data, such as `clone_from_slice()`.
//!
//! ## License: MIT
//!

use std::error;
use std::fmt;

/// Error returned when the wrapped data structure has the wrong dimensions,
/// typically that it is too short.
#[derive(Debug)]
pub struct BufferSizeError {
    desc: String,
}

impl fmt::Display for BufferSizeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.desc)
    }
}

impl error::Error for BufferSizeError {
    fn description(&self) -> &str {
        &self.desc
    }
}

impl BufferSizeError {
    pub fn new(desc: &str) -> Self {
        BufferSizeError {
            desc: desc.to_owned(),
        }
    }
}

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

/// Wrapper for a slice of length `channels`, containing vectors of length `frames`.
/// Each vector contains the samples for all frames of one channel.
pub struct SliceOfChannelVecs<'a, T> {
    buf: &'a mut [Vec<T>],
    frames: usize,
    channels: usize,
}

impl<'a, T> SliceOfChannelVecs<'a, T> {
    /// Create a new `SliceOfChannelVecs` to wrap a slice of vectors.
    /// The slice must contain at least `channels` vectors,
    /// and each vector must be at least `frames` long.
    /// They are allowed to be longer than needed,
    /// but these extra frames or channels cannot
    /// be accessed via the `AudioBuffer` trait methods.
    pub fn new(
        buf: &'a mut [Vec<T>],
        channels: usize,
        frames: usize,
    ) -> Result<Self, BufferSizeError> {
        if buf.len() < channels {
            return Err(BufferSizeError {
                desc: format!("Too few channels, {} < {}", buf.len(), channels),
            });
        }
        for (idx, chan) in buf.iter().enumerate() {
            if chan.len() < frames {
                return Err(BufferSizeError {
                    desc: format!("Channel {} is too short, {} < {}", idx, chan.len(), frames),
                });
            }
        }
        Ok(Self {
            buf,
            frames,
            channels,
        })
    }
}

impl<'a, T> AudioBuffer<'a, T> for SliceOfChannelVecs<'a, T>
where
    T: Clone,
{
    unsafe fn get_unchecked(&self, channel: usize, frame: usize) -> &T {
        return self.buf.get_unchecked(channel).get_unchecked(frame);
    }

    unsafe fn get_unchecked_mut(&mut self, channel: usize, frame: usize) -> &mut T {
        return self.buf.get_unchecked_mut(channel).get_unchecked_mut(frame);
    }

    fn channels(&self) -> usize {
        self.channels
    }

    fn frames(&self) -> usize {
        self.frames
    }

    implement_iterators!();

    fn read_into_channel_from_slice(&mut self, channel: usize, start: usize, slice: &[T]) -> usize {
        if channel >= self.channels || start >= self.frames {
            return 0;
        }
        let frames_to_read = if (self.frames - start) < slice.len() {
            self.frames - start
        } else {
            slice.len()
        };
        self.buf[channel][start..start + frames_to_read].clone_from_slice(&slice[..frames_to_read]);
        frames_to_read
    }

    fn write_from_channel_to_slice(&self, channel: usize, start: usize, slice: &mut [T]) -> usize {
        if channel >= self.channels || start >= self.frames {
            return 0;
        }
        let frames_to_write = if (self.frames - start) < slice.len() {
            self.frames - start
        } else {
            slice.len()
        };
        slice[..frames_to_write]
            .clone_from_slice(&self.buf[channel][start..start + frames_to_write]);
        frames_to_write
    }
}

/// Wrapper for a slice of length `frames`, containing vectors of length `channels`.
/// Each vector contains the samples for all channels of one frame.
pub struct SliceOfFrameVecs<'a, T> {
    buf: &'a mut [Vec<T>],
    frames: usize,
    channels: usize,
}

impl<'a, T> SliceOfFrameVecs<'a, T> {
    /// Create a new `SliceOfFrameVecs` to wrap a slice of vectors.
    /// The slice must contain at least `frames` vectors,
    /// and each vector must be at least `channels` long.
    /// They are allowed to be longer than needed,
    /// but these extra frames or channels cannot
    /// be accessed via the `AudioBuffer` trait methods.
    pub fn new(
        buf: &'a mut [Vec<T>],
        channels: usize,
        frames: usize,
    ) -> Result<Self, BufferSizeError> {
        if buf.len() < frames {
            return Err(BufferSizeError {
                desc: format!("Too few frames, {} < {}", buf.len(), frames),
            });
        }
        for (idx, frame) in buf.iter().enumerate() {
            if frame.len() < channels {
                return Err(BufferSizeError {
                    desc: format!("Frame {} is too short, {} < {}", idx, frame.len(), channels),
                });
            }
        }
        Ok(Self {
            buf,
            frames,
            channels,
        })
    }
}

impl<'a, T> AudioBuffer<'a, T> for SliceOfFrameVecs<'a, T>
where
    T: Clone,
{
    unsafe fn get_unchecked(&self, channel: usize, frame: usize) -> &T {
        return self.buf.get_unchecked(frame).get_unchecked(channel);
    }

    unsafe fn get_unchecked_mut(&mut self, channel: usize, frame: usize) -> &mut T {
        return self.buf.get_unchecked_mut(frame).get_unchecked_mut(channel);
    }

    fn channels(&self) -> usize {
        self.channels
    }

    fn frames(&self) -> usize {
        self.frames
    }

    implement_iterators!();

    fn read_into_frame_from_slice(&mut self, frame: usize, start: usize, slice: &[T]) -> usize {
        if frame >= self.frames || start >= self.channels {
            return 0;
        }
        let channels_to_read = if (self.channels - start) < slice.len() {
            self.channels - start
        } else {
            slice.len()
        };
        self.buf[frame][start..start + channels_to_read]
            .clone_from_slice(&slice[..channels_to_read]);
        channels_to_read
    }

    fn write_from_frame_to_slice(&self, frame: usize, start: usize, slice: &mut [T]) -> usize {
        if frame >= self.frames || start >= self.channels {
            return 0;
        }
        let channels_to_write = if (self.channels - start) < slice.len() {
            self.channels - start
        } else {
            slice.len()
        };
        slice[..channels_to_write]
            .clone_from_slice(&self.buf[frame][start..start + channels_to_write]);
        channels_to_write
    }
}

/// Wrapper for a slice of length `frames * channels`.
/// The samples are stored in _interleaved_ order,
/// where all the samples for one frame are stored consecutively,
/// followed by the samples for the next frame.
/// For a stereo buffer containing four frames, the order is
/// `L1, R1, L2, R2, L3, R3, L4, R4`
pub struct InterleavedSlice<'a, T> {
    buf: &'a mut [T],
    frames: usize,
    channels: usize,
}

impl<'a, T> InterleavedSlice<'a, T> {
    /// Create a new `InterleavedSlice` to wrap a slice.
    /// The slice length must be at least `frames*channels`.
    /// It is allowed to be longer than needed,
    /// but these extra values cannot
    /// be accessed via the `AudioBuffer` trait methods.
    pub fn new(buf: &'a mut [T], channels: usize, frames: usize) -> Result<Self, BufferSizeError> {
        if buf.len() < frames * channels {
            return Err(BufferSizeError {
                desc: format!("Buffer is too short, {} < {}", buf.len(), frames * channels),
            });
        }
        Ok(Self {
            buf,
            frames,
            channels,
        })
    }
}

impl<'a, T> AudioBuffer<'a, T> for InterleavedSlice<'a, T>
where
    T: Clone,
{
    unsafe fn get_unchecked(&self, channel: usize, frame: usize) -> &T {
        return self.buf.get_unchecked(frame * self.channels + channel);
    }

    unsafe fn get_unchecked_mut(&mut self, channel: usize, frame: usize) -> &mut T {
        return self.buf.get_unchecked_mut(frame * self.channels + channel);
    }

    fn channels(&self) -> usize {
        self.channels
    }

    fn frames(&self) -> usize {
        self.frames
    }

    implement_iterators!();

    fn read_into_frame_from_slice(&mut self, frame: usize, start: usize, slice: &[T]) -> usize {
        if frame >= self.frames || start >= self.channels {
            return 0;
        }
        let channels_to_read = if (self.channels - start) < slice.len() {
            self.channels - start
        } else {
            slice.len()
        };
        let buffer_start = start + frame * self.channels;
        self.buf[buffer_start..buffer_start + channels_to_read]
            .clone_from_slice(&slice[..channels_to_read]);
        channels_to_read
    }

    fn write_from_frame_to_slice(&self, frame: usize, start: usize, slice: &mut [T]) -> usize {
        if frame >= self.frames || start >= self.channels {
            return 0;
        }
        let channels_to_write = if (self.channels - start) < slice.len() {
            self.channels - start
        } else {
            slice.len()
        };
        let buffer_start = start + frame * self.channels;
        slice[..channels_to_write]
            .clone_from_slice(&self.buf[buffer_start..buffer_start + channels_to_write]);
        channels_to_write
    }
}

/// Wrapper for a slice of length `frames * channels`.
/// The samples are stored in _sequential_ order,
/// where all the samples for one channel are stored consecutively,
/// followed by the samples for the next channel.
/// For a stereo buffer containing four frames, the order is
/// `L1, L2, L3, L4, R1, R2, R3, R4`
pub struct SequentialSlice<'a, T> {
    buf: &'a mut [T],
    frames: usize,
    channels: usize,
}

impl<'a, T> SequentialSlice<'a, T> {
    /// Create a new `SequentialSlice` to wrap a slice.
    /// The slice length must be at least `frames*channels`.
    /// It is allowed to be longer than needed,
    /// but these extra values cannot
    /// be accessed via the `AudioBuffer` trait methods.
    pub fn new(buf: &'a mut [T], channels: usize, frames: usize) -> Result<Self, BufferSizeError> {
        if buf.len() < frames * channels {
            return Err(BufferSizeError {
                desc: format!("Buffer is too short, {} < {}", buf.len(), frames * channels),
            });
        }
        Ok(Self {
            buf,
            frames,
            channels,
        })
    }
}

impl<'a, T> AudioBuffer<'a, T> for SequentialSlice<'a, T>
where
    T: Clone,
{
    unsafe fn get_unchecked(&self, channel: usize, frame: usize) -> &T {
        return self.buf.get_unchecked(channel * self.frames + frame);
    }

    unsafe fn get_unchecked_mut(&mut self, channel: usize, frame: usize) -> &mut T {
        return self.buf.get_unchecked_mut(channel * self.frames + frame);
    }

    fn channels(&self) -> usize {
        self.channels
    }

    fn frames(&self) -> usize {
        self.frames
    }

    implement_iterators!();

    fn read_into_channel_from_slice(&mut self, channel: usize, start: usize, slice: &[T]) -> usize {
        if channel >= self.channels || start >= self.frames {
            return 0;
        }
        let frames_to_read = if (self.frames - start) < slice.len() {
            self.frames - start
        } else {
            slice.len()
        };
        let buffer_start = channel * self.frames + start;
        self.buf[buffer_start..buffer_start + frames_to_read]
            .clone_from_slice(&slice[..frames_to_read]);
        frames_to_read
    }

    fn write_from_channel_to_slice(&self, channel: usize, start: usize, slice: &mut [T]) -> usize {
        if channel >= self.channels || start >= self.frames {
            return 0;
        }
        let frames_to_write = if (self.frames - start) < slice.len() {
            self.frames - start
        } else {
            slice.len()
        };
        let buffer_start = channel * self.frames + start;
        slice[..frames_to_write]
            .clone_from_slice(&self.buf[buffer_start..buffer_start + frames_to_write]);
        frames_to_write
    }
}

// -------------------- The main buffer trait --------------------

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

    /// Get the number of channels stored in this `AudioBuffer`.
    fn channels(&self) -> usize;

    /// Get the number of frames stored in this `AudioBuffer`.
    fn frames(&self) -> usize;

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
        for n in 0..frames_to_read {
            unsafe { *self.get_unchecked_mut(channel, start + n) = slice[n].clone() };
        }
        frames_to_read
    }

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
        for n in 0..frames_to_write {
            unsafe { slice[n] = self.get_unchecked(channel, start + n).clone() };
        }
        frames_to_write
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
        for n in 0..channels_to_read {
            unsafe { *self.get_unchecked_mut(start + n, frame) = slice[n].clone() };
        }
        channels_to_read
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
        for n in 0..channels_to_write {
            unsafe { slice[n] = self.get_unchecked(start + n, frame).clone() };
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

// -------------------- Iterators returning immutable samples --------------------

/// An iterator that yields immutable references to the samples of a channel.
pub struct ChannelSamples<'a, 'b, T> {
    buf: &'b dyn AudioBuffer<'a, T>,
    frame: usize,
    nbr_frames: usize,
    channel: usize,
}

impl<'a, 'b, T> ChannelSamples<'a, 'b, T>
where
    T: Clone,
{
    pub fn new(
        buffer: &'b dyn AudioBuffer<'a, T>,
        channel: usize,
    ) -> Option<ChannelSamples<'a, 'b, T>> {
        if channel >= buffer.channels() {
            return None;
        }
        let nbr_frames = buffer.frames();
        Some(ChannelSamples {
            buf: buffer as &'b dyn AudioBuffer<'a, T>,
            frame: 0,
            nbr_frames,
            channel,
        })
    }
}

impl<'a, 'b, T> Iterator for ChannelSamples<'a, 'b, T>
where
    T: Clone,
{
    type Item = &'b T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.frame >= self.nbr_frames {
            return None;
        }
        let val = unsafe { self.buf.get_unchecked(self.channel, self.frame) };
        self.frame += 1;
        Some(val)
    }
}

/// An iterator that yields immutable references to the samples of a frame.
pub struct FrameSamples<'a, 'b, T> {
    buf: &'b dyn AudioBuffer<'a, T>,
    frame: usize,
    nbr_channels: usize,
    channel: usize,
}

impl<'a, 'b, T> FrameSamples<'a, 'b, T>
where
    T: Clone,
{
    pub fn new(
        buffer: &'b dyn AudioBuffer<'a, T>,
        frame: usize,
    ) -> Option<FrameSamples<'a, 'b, T>> {
        if frame >= buffer.frames() {
            return None;
        }
        let nbr_channels = buffer.channels();
        Some(FrameSamples {
            buf: buffer as &'b dyn AudioBuffer<'a, T>,
            channel: 0,
            nbr_channels,
            frame,
        })
    }
}

impl<'a, 'b, T> Iterator for FrameSamples<'a, 'b, T>
where
    T: Clone,
{
    type Item = &'b T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.channel >= self.nbr_channels {
            return None;
        }
        let val = unsafe { self.buf.get_unchecked(self.channel, self.frame) };
        self.channel += 1;
        Some(val)
    }
}

// -------------------- Iterators returning immutable iterators --------------------

/// An iterator that yields a [ChannelSamples] iterator for each channel of an [AudioBuffer].
pub struct Channels<'a, 'b, T> {
    buf: &'b dyn AudioBuffer<'a, T>,
    nbr_channels: usize,
    channel: usize,
}

impl<'a, 'b, T> Channels<'a, 'b, T>
where
    T: Clone,
{
    pub fn new(buffer: &'b dyn AudioBuffer<'a, T>) -> Channels<'a, 'b, T> {
        let nbr_channels = buffer.channels();
        Channels {
            buf: buffer as &'b dyn AudioBuffer<'a, T>,
            channel: 0,
            nbr_channels,
        }
    }
}

impl<'a, 'b, T> Iterator for Channels<'a, 'b, T>
where
    T: Clone,
{
    type Item = ChannelSamples<'a, 'b, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.channel >= self.nbr_channels {
            return None;
        }
        let val = ChannelSamples::new(self.buf, self.channel).unwrap();
        self.channel += 1;
        Some(val)
    }
}

/// An iterator that yields a [FrameSamples] iterator for each frame of an [AudioBuffer].
pub struct Frames<'a, 'b, T> {
    buf: &'b dyn AudioBuffer<'a, T>,
    nbr_frames: usize,
    frame: usize,
}

impl<'a, 'b, T> Frames<'a, 'b, T>
where
    T: Clone,
{
    pub fn new(buffer: &'b dyn AudioBuffer<'a, T>) -> Frames<'a, 'b, T> {
        let nbr_frames = buffer.frames();
        Frames {
            buf: buffer as &'b dyn AudioBuffer<'a, T>,
            frame: 0,
            nbr_frames,
        }
    }
}

impl<'a, 'b, T> Iterator for Frames<'a, 'b, T>
where
    T: Clone,
{
    type Item = FrameSamples<'a, 'b, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.frame >= self.nbr_frames {
            return None;
        }
        let val = FrameSamples::new(self.buf, self.frame).unwrap();
        self.frame += 1;
        Some(val)
    }
}

// -------------------- Iterators returning mutable samples --------------------

/// An iterator that yields mutable references to the samples of a channel.
pub struct ChannelSamplesMut<'a, 'b, T> {
    buf: &'b mut dyn AudioBuffer<'a, T>,
    frame: usize,
    nbr_frames: usize,
    channel: usize,
}

impl<'a, 'b, T> ChannelSamplesMut<'a, 'b, T>
where
    T: Clone,
{
    pub fn new(
        buffer: &'b mut dyn AudioBuffer<'a, T>,
        channel: usize,
    ) -> Option<ChannelSamplesMut<'a, 'b, T>> {
        if channel >= buffer.channels() {
            return None;
        }
        let nbr_frames = buffer.frames();
        Some(ChannelSamplesMut {
            buf: buffer as &'b mut dyn AudioBuffer<'a, T>,
            frame: 0,
            nbr_frames,
            channel,
        })
    }
}

impl<'a, 'b, T> Iterator for ChannelSamplesMut<'a, 'b, T>
where
    T: Clone,
{
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.frame >= self.nbr_frames {
            return None;
        }
        let val = unsafe { self.buf.get_unchecked_mut(self.channel, self.frame) };
        // The compiler doesn't know that the iterator never returns the same value twice.
        // Therefore it will not let us return a mutable reference with lifetime 'a.
        // Go via a raw pointer to bypass this.
        let val_ptr = val as *mut T;
        let return_val = unsafe { &mut *val_ptr };
        self.frame += 1;
        Some(return_val)
    }
}

/// An iterator that yields mutable references to the samples of a frame.
pub struct FrameSamplesMut<'a, 'b, T> {
    buf: &'b mut dyn AudioBuffer<'a, T>,
    frame: usize,
    nbr_channels: usize,
    channel: usize,
}

impl<'a, 'b, T> FrameSamplesMut<'a, 'b, T>
where
    T: Clone,
{
    pub fn new(
        buffer: &'b mut dyn AudioBuffer<'a, T>,
        frame: usize,
    ) -> Option<FrameSamplesMut<'a, 'b, T>> {
        if frame >= buffer.frames() {
            return None;
        }
        let nbr_channels = buffer.channels();
        Some(FrameSamplesMut {
            buf: buffer as &'b mut dyn AudioBuffer<'a, T>,
            channel: 0,
            nbr_channels,
            frame,
        })
    }
}

impl<'a, 'b, T> Iterator for FrameSamplesMut<'a, 'b, T>
where
    T: Clone,
{
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.channel >= self.nbr_channels {
            return None;
        }
        let val = unsafe { self.buf.get_unchecked_mut(self.channel, self.frame) };
        // The compiler doesn't know that the iterator never returns the same value twice.
        // Therefore it will not let us return a mutable reference with lifetime 'a.
        // Go via a raw pointer to bypass this.
        let val_ptr = val as *mut T;
        let return_val = unsafe { &mut *val_ptr };
        self.channel += 1;
        Some(return_val)
    }
}

// -------------------- Iterators returning mutable iterators --------------------

/// An iterator that yields a [ChannelSamplesMut] iterator for each channel of an [AudioBuffer].
pub struct ChannelsMut<'a, 'b, T> {
    buf: &'b mut dyn AudioBuffer<'a, T>,
    nbr_channels: usize,
    channel: usize,
}

impl<'a, 'b, T> ChannelsMut<'a, 'b, T>
where
    T: Clone,
{
    pub fn new(buffer: &'b mut dyn AudioBuffer<'a, T>) -> ChannelsMut<'a, 'b, T> {
        let nbr_channels = buffer.channels();
        ChannelsMut {
            buf: buffer as &'b mut dyn AudioBuffer<'a, T>,
            channel: 0,
            nbr_channels,
        }
    }
}

impl<'a, 'b, T> Iterator for ChannelsMut<'a, 'b, T>
where
    T: Clone,
{
    type Item = ChannelSamplesMut<'a, 'b, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.channel >= self.nbr_channels {
            return None;
        }
        // The compiler doesn't know that the iterator never returns the same value twice.
        // Therefore it will not let us return a mutable reference with lifetime 'a.
        // Go via a raw pointer to bypass this.
        let buf_ptr = self.buf as *mut dyn AudioBuffer<'a, T>;
        let return_buf = unsafe { &mut *buf_ptr };
        let val = ChannelSamplesMut::new(return_buf, self.channel).unwrap();
        self.channel += 1;
        Some(val)
    }
}

/// An iterator that yields a [FrameSamplesMut] iterator for each frame of an [AudioBuffer].
pub struct FramesMut<'a, 'b, T> {
    buf: &'b mut dyn AudioBuffer<'a, T>,
    nbr_frames: usize,
    frame: usize,
}

impl<'a, 'b, T> FramesMut<'a, 'b, T>
where
    T: Clone,
{
    pub fn new(buffer: &'b mut dyn AudioBuffer<'a, T>) -> FramesMut<'a, 'b, T> {
        let nbr_frames = buffer.frames();
        FramesMut {
            buf: buffer as &'b mut dyn AudioBuffer<'a, T>,
            frame: 0,
            nbr_frames,
        }
    }
}

impl<'a, 'b, T> Iterator for FramesMut<'a, 'b, T>
where
    T: Clone,
{
    type Item = FrameSamplesMut<'a, 'b, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.frame >= self.nbr_frames {
            return None;
        }
        // The compiler doesn't know that the iterator never returns the same value twice.
        // Therefore it will not let us return a mutable reference with lifetime 'a.
        // Go via a raw pointer to bypass this.
        let buf_ptr = self.buf as *mut dyn AudioBuffer<'a, T>;
        let return_buf = unsafe { &mut *buf_ptr };
        let val = FrameSamplesMut::new(return_buf, self.frame).unwrap();
        self.frame += 1;
        Some(val)
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

    fn insert_data(buffer: &mut dyn AudioBuffer<i32>) {
        *buffer.get_mut(0, 0).unwrap() = 1;
        *buffer.get_mut(0, 1).unwrap() = 2;
        *buffer.get_mut(0, 2).unwrap() = 3;
        *buffer.get_mut(1, 0).unwrap() = 4;
        *buffer.get_mut(1, 1).unwrap() = 5;
        *buffer.get_mut(1, 2).unwrap() = 6;
    }

    fn test_get(buffer: &mut dyn AudioBuffer<i32>) {
        insert_data(buffer);
        assert_eq!(*buffer.get(0, 0).unwrap(), 1);
        assert_eq!(*buffer.get(0, 1).unwrap(), 2);
        assert_eq!(*buffer.get(0, 2).unwrap(), 3);
        assert_eq!(*buffer.get(1, 0).unwrap(), 4);
        assert_eq!(*buffer.get(1, 1).unwrap(), 5);
        assert_eq!(*buffer.get(1, 2).unwrap(), 6);
    }

    fn test_iter(buffer: &mut dyn AudioBuffer<i32>) {
        insert_data(buffer);
        let mut iter1 = buffer.iter_channel(0).unwrap();
        assert_eq!(iter1.next(), Some(&1));
        assert_eq!(iter1.next(), Some(&2));
        assert_eq!(iter1.next(), Some(&3));
        assert_eq!(iter1.next(), None);

        let mut iter2 = buffer.iter_frame(1).unwrap();
        assert_eq!(iter2.next(), Some(&2));
        assert_eq!(iter2.next(), Some(&5));
        assert_eq!(iter2.next(), None);
    }

    fn test_iter_mut(buffer: &mut dyn AudioBuffer<i32>) {
        insert_data(buffer);
        let mut sum = 0;
        for channel in buffer.iter_channels() {
            sum += channel.sum::<i32>();
        }
        assert_eq!(sum, 21);

        for channel in buffer.iter_channels_mut() {
            for sample in channel {
                *sample = 2 * *sample;
            }
        }
        let mut sum = 0;
        for channel in buffer.iter_channels() {
            sum += channel.sum::<i32>();
        }
        assert_eq!(sum, 42);
    }

    fn test_slice_channel(buffer: &mut dyn AudioBuffer<i32>) {
        insert_data(buffer);
        let mut other1 = vec![0; 2];
        let mut other2 = vec![0; 4];
        buffer.write_from_channel_to_slice(0, 1, &mut other1);
        buffer.write_from_channel_to_slice(1, 0, &mut other2);
        assert_eq!(other1[0], 2);
        assert_eq!(other1[1], 3);
        assert_eq!(other2[0], 4);
        assert_eq!(other2[1], 5);
        assert_eq!(other2[2], 6);
        assert_eq!(other2[3], 0);
    }

    fn test_slice_frame(buffer: &mut dyn AudioBuffer<i32>) {
        insert_data(buffer);
        let mut other1 = vec![0; 1];
        let mut other2 = vec![0; 3];
        buffer.write_from_frame_to_slice(0, 1, &mut other1);
        buffer.write_from_frame_to_slice(1, 0, &mut other2);
        assert_eq!(other1[0], 4);
        assert_eq!(other2[0], 2);
        assert_eq!(other2[1], 5);
        assert_eq!(other2[2], 0);
    }

    fn test_mut_slice_channel(buffer: &mut dyn AudioBuffer<i32>) {
        insert_data(buffer);
        let other1 = vec![8, 9];
        let other2 = vec![10, 11, 12, 13];
        buffer.read_into_channel_from_slice(0, 1, &other1);
        buffer.read_into_channel_from_slice(1, 0, &other2);
        assert_eq!(*buffer.get(0, 0).unwrap(), 1);
        assert_eq!(*buffer.get(0, 1).unwrap(), 8);
        assert_eq!(*buffer.get(0, 2).unwrap(), 9);
        assert_eq!(*buffer.get(1, 0).unwrap(), 10);
        assert_eq!(*buffer.get(1, 1).unwrap(), 11);
        assert_eq!(*buffer.get(1, 2).unwrap(), 12);
    }

    fn test_mut_slice_frame(buffer: &mut dyn AudioBuffer<i32>) {
        insert_data(buffer);
        let other1 = vec![8];
        let other2 = vec![10, 11, 12];
        buffer.read_into_frame_from_slice(0, 0, &other1);
        buffer.read_into_frame_from_slice(1, 0, &other2);
        assert_eq!(*buffer.get(0, 0).unwrap(), 8);
        assert_eq!(*buffer.get(1, 0).unwrap(), 4);
        assert_eq!(*buffer.get(0, 1).unwrap(), 10);
        assert_eq!(*buffer.get(1, 1).unwrap(), 11);
        assert_eq!(*buffer.get(0, 2).unwrap(), 3);
        assert_eq!(*buffer.get(1, 2).unwrap(), 6);
    }

    #[test]
    fn vec_of_channels() {
        let mut data = vec![vec![0_i32; 3], vec![0_i32; 3]];
        let mut buffer = SliceOfChannelVecs::new(&mut data, 2, 3).unwrap();
        test_get(&mut buffer);
        test_iter(&mut buffer);
        test_iter_mut(&mut buffer);
        test_slice_channel(&mut buffer);
        test_slice_frame(&mut buffer);
        test_mut_slice_channel(&mut buffer);
        test_mut_slice_frame(&mut buffer);
    }

    #[test]
    fn vec_of_frames() {
        let mut data = vec![vec![1_i32, 4], vec![2_i32, 5], vec![3, 6]];
        let mut buffer = SliceOfFrameVecs::new(&mut data, 2, 3).unwrap();
        test_get(&mut buffer);
        test_iter(&mut buffer);
        test_iter_mut(&mut buffer);
        test_slice_channel(&mut buffer);
        test_slice_frame(&mut buffer);
        test_mut_slice_channel(&mut buffer);
        test_mut_slice_frame(&mut buffer);
    }

    #[test]
    fn interleaved() {
        let mut data = vec![1_i32, 4, 2, 5, 3, 6];
        let mut buffer = InterleavedSlice::new(&mut data, 2, 3).unwrap();
        test_get(&mut buffer);
        test_iter(&mut buffer);
        test_iter_mut(&mut buffer);
        test_slice_channel(&mut buffer);
        test_slice_frame(&mut buffer);
        test_mut_slice_channel(&mut buffer);
        test_mut_slice_frame(&mut buffer);
    }

    #[test]
    fn sequential() {
        let mut data = vec![1_i32, 2, 3, 4, 5, 6];
        let mut buffer = SequentialSlice::new(&mut data, 2, 3).unwrap();
        test_get(&mut buffer);
        test_iter(&mut buffer);
        test_iter_mut(&mut buffer);
        test_slice_channel(&mut buffer);
        test_slice_frame(&mut buffer);
        test_mut_slice_channel(&mut buffer);
        test_mut_slice_frame(&mut buffer);
    }

    // This tests that an AudioBuffer is object safe.
    #[test]
    fn boxed_buffer() {
        let mut data = vec![1_i32, 2, 3, 4, 5, 6];
        let boxed: Box<dyn AudioBuffer<i32>> =
            Box::new(SequentialSlice::new(&mut data, 2, 3).unwrap());
        assert_eq!(*boxed.get(0, 0).unwrap(), 1);
    }
}
