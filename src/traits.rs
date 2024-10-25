//! # audioadapter traits
//!
//! A set of traits for making it easier to work with buffers of audio data.

// -------------------- The main buffer trait --------------------

/// A trait for reading samples from a buffer.
/// Samples accessed indirectly by a `read` method.
/// Implementations may perform any needed transformation
/// of the sample value before returning it.
pub trait Adapter<'a, T: 'a> {
    /// Read the sample at
    /// a given combination of frame and channel.
    ///
    /// # Safety
    ///
    /// This method performs no bounds checking.
    /// Calling it with an out-of-bound value for frame or channel
    /// results in undefined behavior,
    /// for example returning an invalid value or panicking.
    unsafe fn read_sample_unchecked(&self, channel: usize, frame: usize) -> T;

    /// Read the sample at
    /// a given combination of frame and channel.
    /// Returns `None` if the frame or channel is
    /// out of bounds of the buffer.
    fn read_sample(&self, channel: usize, frame: usize) -> Option<T> {
        if channel >= self.channels() || frame >= self.frames() {
            return None;
        }
        Some(unsafe { self.read_sample_unchecked(channel, frame) })
    }

    /// Get the number of channels stored in this buffer.
    fn channels(&self) -> usize;

    /// Get the number of frames stored in this buffer.
    fn frames(&self) -> usize;

    /// Write values from a channel of the buffer to a slice.
    /// The `skip` argument is the offset into the buffer channel
    /// where the first value will be read from.
    /// If the slice is longer than the available number of values in the channel of the buffer,
    /// then only the available number of samples will be written.
    ///
    /// Returns the number of values written.
    /// If an invalid channel number is given,
    /// or if `skip` is larger than the length of the channel,
    /// no samples will be written and zero is returned.
    fn write_from_channel_to_slice(&self, channel: usize, skip: usize, slice: &mut [T]) -> usize {
        if channel >= self.channels() || skip >= self.frames() {
            return 0;
        }
        let frames_to_write = if (self.frames() - skip) < slice.len() {
            self.frames() - skip
        } else {
            slice.len()
        };
        for (n, item) in slice.iter_mut().enumerate().take(frames_to_write) {
            unsafe { *item = self.read_sample_unchecked(channel, skip + n) };
        }
        frames_to_write
    }

    /// Write values from a frame of the buffer to a slice.
    /// The `skip` argument is the offset into the buffer frame
    /// where the first value will be read from.
    /// If the slice is longer than the available number of values in the buffer frame,
    /// then only the available number of samples will be written.
    ///
    /// Returns the number of values written.
    /// If an invalid frame number is given,
    /// or if `skip` is larger than the length of the frame,
    /// no samples will be written and zero is returned.
    fn write_from_frame_to_slice(&self, frame: usize, skip: usize, slice: &mut [T]) -> usize {
        if frame >= self.frames() || skip >= self.channels() {
            return 0;
        }
        let channels_to_write = if (self.channels() - skip) < slice.len() {
            self.channels() - skip
        } else {
            slice.len()
        };
        for (n, item) in slice.iter_mut().enumerate().take(channels_to_write) {
            unsafe { *item = self.read_sample_unchecked(skip + n, frame) };
        }
        channels_to_write
    }
}

/// A trait for writing samples to a buffer.
/// Samples are accessed indirectly by a `write` method.
/// Implementations may perform any needed transformation
/// of the sample value before writing to the underlying buffer.
pub trait AdapterMut<'a, T>: Adapter<'a, T>
where
    T: Clone + 'a,
{
    /// Write a sample to the
    /// given combination of frame and channel.
    /// Returns a boolean indicating if the sample value
    /// was clipped during conversion.
    /// Implementations that do not perform any conversion
    /// always return `false`.
    ///
    /// # Safety
    ///
    /// This method performs no bounds checking.
    /// Calling it with an out-of-bound value for frame or channel
    /// results in undefined behavior,
    /// for example returning an invalid value or panicking.
    unsafe fn write_sample_unchecked(&mut self, channel: usize, frame: usize, value: &T) -> bool;

    /// Write a sample to the
    /// given combination of frame and channel.
    /// Returns a boolean indicating if the sample value
    /// was clipped during conversion.
    /// Implementations that do not perform any conversion
    /// always return `false`.
    /// Returns `None` if the frame or channel is
    /// out of bounds of the buffer.
    fn write_sample(&mut self, channel: usize, frame: usize, value: &T) -> Option<bool> {
        if channel >= self.channels() || frame >= self.frames() {
            return None;
        }
        Some(unsafe { self.write_sample_unchecked(channel, frame, value) })
    }

    /// Write values from a slice into a channel of the buffer.
    /// The `skip` argument is the offset into the buffer channel
    /// where the first value will be written.
    /// If the slice is longer than the available space in the buffer channel,
    /// then only the number of samples that fit will be read.
    ///
    /// Returns a tuple of two numbers.
    /// The first is the number of values written,
    /// and the second is the number of values that were clipped during conversion.
    /// Implementations that do not perform any conversion
    /// always return zero clipped samples.
    /// If an invalid channel number is given,
    /// or if `skip` is larger than the length of the channel,
    /// no samples will be read and (0, 0) is returned.
    fn write_from_slice_to_channel(
        &mut self,
        channel: usize,
        skip: usize,
        slice: &[T],
    ) -> (usize, usize) {
        if channel >= self.channels() || skip >= self.frames() {
            return (0, 0);
        }
        let frames_to_read = if (self.frames() - skip) < slice.len() {
            self.frames() - skip
        } else {
            slice.len()
        };
        let mut nbr_clipped = 0;
        for (n, item) in slice.iter().enumerate().take(frames_to_read) {
            unsafe { nbr_clipped += self.write_sample_unchecked(channel, skip + n, item) as usize };
        }
        (frames_to_read, nbr_clipped)
    }

    /// Write values from a slice into a frame of the buffer.
    /// The `skip` argument is the offset into the buffer frame
    /// where the first value will be written.
    /// If the slice is longer than the available space in the buffer frame,
    /// then only the number of samples that fit will be read.
    ///
    /// Returns a tuple of two numbers.
    /// The first is the number of values written,
    /// and the second is the number of values that were clipped during conversion.
    /// Implementations that do not perform any conversion
    /// always return zero clipped samples.
    /// If an invalid frame number is given,
    /// or if `skip` is larger than the length of the frame,
    /// no samples will be read and (0, 0) is returned.
    fn write_from_slice_to_frame(
        &mut self,
        frame: usize,
        skip: usize,
        slice: &[T],
    ) -> (usize, usize) {
        if frame >= self.frames() || skip >= self.channels() {
            return (0, 0);
        }
        let channels_to_read = if (self.channels() - skip) < slice.len() {
            self.channels() - skip
        } else {
            slice.len()
        };
        let mut nbr_clipped = 0;
        for (n, item) in slice.iter().enumerate().take(channels_to_read) {
            unsafe { nbr_clipped += self.write_sample_unchecked(skip + n, frame, item) as usize };
        }
        (channels_to_read, nbr_clipped)
    }

    /// Copy values from a channel of another buffer to self.
    /// The `self_skip` and `other_skip` arguments are the offsets
    /// in frames for where copying starts in the two buffers.
    /// The method copies `take` values.
    ///
    /// Returns the the number of values that were clipped during conversion.
    /// Implementations that do not perform any conversion
    /// always return zero clipped samples.
    ///
    /// If an invalid channel number is given,
    /// or if either of the buffers is to short to copy `take` values,
    /// no values will be copied and `None` is returned.
    fn write_from_other_to_channel(
        &mut self,
        other: &dyn Adapter<'a, T>,
        other_channel: usize,
        self_channel: usize,
        other_skip: usize,
        self_skip: usize,
        take: usize,
    ) -> Option<usize> {
        if self_channel >= self.channels()
            || take + self_skip > self.frames()
            || other_channel >= other.channels()
            || take + other_skip > other.frames()
        {
            return None;
        }
        let mut nbr_clipped = 0;
        for n in 0..take {
            unsafe {
                let value = other.read_sample_unchecked(other_channel, n + other_skip);
                nbr_clipped +=
                    self.write_sample_unchecked(self_channel, n + self_skip, &value) as usize
            };
        }
        Some(nbr_clipped)
    }

    /// Write the provided value to every sample in a channel.
    /// Can be used to clear a channel by writing zeroes,
    /// or to initialize each sample to a certain value.
    /// Returns `None` if called with an invalid channel number.
    fn fill_channel_with(&mut self, channel: usize, value: &T) -> Option<()> {
        if channel >= self.channels() {
            return None;
        }
        for frame in 0..self.frames() {
            unsafe { self.write_sample_unchecked(channel, frame, value) };
        }
        Some(())
    }

    /// Write the provided value to every sample in a frame.
    /// Can be used to clear a frame by writing zeroes,
    /// or to initialize each sample to a certain value.
    /// Returns `None` if called with an invalid frame number.
    fn fill_frame_with(&mut self, frame: usize, value: &T) -> Option<()> {
        if frame >= self.frames() {
            return None;
        }
        for channel in 0..self.channels() {
            unsafe { self.write_sample_unchecked(channel, frame, value) };
        }
        Some(())
    }

    /// Write the provided value to every sample in a range of frames.
    /// Can be used to clear a range of frames by writing zeroes,
    /// or to initialize each sample to a certain value.
    /// Returns `None` if called with a too large range.
    fn fill_frames_with(&mut self, start: usize, count: usize, value: &T) -> Option<usize> {
        if start + count >= self.frames() {
            return None;
        }
        for channel in 0..self.channels() {
            for frame in start..start + count {
                unsafe { self.write_sample_unchecked(channel, frame, value) };
            }
        }
        Some(count)
    }

    /// Write the provided value to every sample in the entire buffer.
    /// Can be used to clear a buffer by writing zeroes,
    /// or to initialize each sample to a certain value.
    fn fill_with(&mut self, value: &T) {
        for channel in 0..self.channels() {
            self.fill_channel_with(channel, value).unwrap_or_default();
        }
    }

    /// Copy frames within the buffer.
    /// Copying is performed for all channels.
    /// Copies (by cloning) `count` frames, from the range `src..src+count`,
    /// to the range `dest..dest+count`.
    /// The two regions are allowed to overlap.
    fn copy_frames_within(&mut self, src: usize, dest: usize, count: usize) -> Option<usize> {
        if src + count > self.frames() || dest + count > self.frames() {
            return None;
        }
        if count == 0 || src == dest {
            return Some(count);
        }
        // This generic implementation is slow, overriding is recommended.
        if dest < src {
            for channel in 0..self.channels() {
                // iterate forward
                for frame in 0..count {
                    unsafe {
                        let value = self.read_sample_unchecked(channel, frame + src);
                        self.write_sample_unchecked(channel, frame + dest, &value);
                    }
                }
            }
        } else {
            for channel in 0..self.channels() {
                // iterate backwards
                for frame in 0..count {
                    let backwards = count - frame - 1;
                    unsafe {
                        let value = self.read_sample_unchecked(channel, backwards + src);
                        self.write_sample_unchecked(channel, backwards + dest, &value);
                    }
                }
            }
        }
        Some(count)
    }
}
