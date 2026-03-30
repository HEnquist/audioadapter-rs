//! # audioadapter traits
//!
//! A set of traits for making it easier to work with buffers of audio data.

// -------------------- The main buffer trait --------------------

/// A trait for reading samples from a buffer.
/// Samples are accessed indirectly by a `read_sample` method.
/// Implementations may perform any needed transformation
/// of the sample value before returning it.
///
/// # Safety
///
/// This trait is `unsafe` because it relies on correct implementations of the
/// `frames` and `channels` methods.
/// Users of an `Adapter` trait object must be able to rely on these values
/// being constant while the buffer is in use.
/// The values returned by these methods are not allowed to change spontaneously,
/// for example due to some internal logic or because of some action in another thread.
///
/// The provided safe helper methods (such as `read_sample` and the copy methods)
/// use these bounds before calling `read_sample_unchecked` internally.
/// If the reported bounds are wrong, those internal unchecked accesses can become
/// out of bounds and cause undefined behavior.
pub unsafe trait Adapter<'a, T: 'a> {
    /// Read the sample at
    /// a given combination of frame and channel.
    ///
    /// # Safety
    ///
    /// This method performs no bounds checking.
    /// The caller must ensure that `channel < self.channels()` and
    /// `frame < self.frames()`.
    /// Violating these preconditions causes undefined behavior.
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

    /// Copy values from a channel of self to a slice.
    /// The `skip` argument is the offset in samples from
    /// where the first value will be copied.
    /// If the target slice is longer than the available number of values in the channel,
    /// then only the available number of samples will be copied.
    ///
    /// Returns the number of values copied.
    /// If an invalid channel number is given,
    /// or if `skip` is larger than the length of the channel,
    /// no samples will be copied and zero is returned.
    fn copy_from_channel_to_slice(&self, channel: usize, skip: usize, slice: &mut [T]) -> usize {
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

    /// Copy values from a frame of self to a slice.
    /// The `skip` argument is the offset in samples from
    /// where the first value will be copied.
    /// If the slice is longer than the available number of values in the frame,
    /// then only the available number of samples will be copied.
    ///
    /// Returns the number of values copied.
    /// If an invalid frame number is given,
    /// or if `skip` is larger than the length of the frame,
    /// no samples will be copied and zero is returned.
    fn copy_from_frame_to_slice(&self, frame: usize, skip: usize, slice: &mut [T]) -> usize {
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
/// Samples are accessed indirectly by a `write_sample` method.
/// Implementations may perform any needed transformation
/// of the sample value before writing to the underlying buffer.
///
/// # Safety
///
/// This trait is `unsafe` because it relies on correct implementations of the
/// `channels` and `frames` methods.
/// The values returned by these methods are not allowed to change spontaneously,
/// for example due to some internal logic or because of some action in another thread.
///
/// The provided safe helper methods (such as `write_sample` and the copy/fill methods)
/// use these bounds before calling `write_sample_unchecked` internally.
/// If the reported bounds are wrong, those internal unchecked accesses can become
/// out of bounds and cause undefined behavior.
pub unsafe trait AdapterMut<'a, T>: Adapter<'a, T>
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
    /// The caller must ensure that `channel < self.channels()` and
    /// `frame < self.frames()`.
    /// Violating these preconditions causes undefined behavior.
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

    /// Copies values from a slice into a channel of self.
    /// The `skip` argument is the offset into the channel to
    /// where the first value will be copied.
    /// If the slice is longer than the available space in the channel,
    /// then only the number of samples that fit will be copied.
    ///
    /// Returns a tuple of two numbers.
    /// The first is the number of values copied,
    /// and the second is the number of values that were clipped during conversion.
    /// Implementations that do not perform any conversion
    /// always return zero clipped samples.
    /// If an invalid channel number is given,
    /// or if `skip` is larger than the length of the channel,
    /// no samples will be copied and (0, 0) is returned.
    fn copy_from_slice_to_channel(
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

    /// Copy values from a slice into a frame of self.
    /// The `skip` argument is the offset into the frame to
    /// where the first value will be copied.
    /// If the slice is longer than the available space in the frame,
    /// then only the number of samples that fit will be copied.
    ///
    /// Returns a tuple of two numbers.
    /// The first is the number of values copied,
    /// and the second is the number of values that were clipped during conversion.
    /// Implementations that do not perform any conversion
    /// always return zero clipped samples.
    /// If an invalid frame number is given,
    /// or if `skip` is larger than the length of the frame,
    /// no samples will be copied and (0, 0) is returned.
    fn copy_from_slice_to_frame(
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

    /// Copy values from a channel of another Adapter.
    /// The `self_skip` and `other_skip` arguments are the offsets
    /// in frames for where copying starts in the two channels.
    /// The method copies `take` values.
    ///
    /// Returns the the number of values that were clipped during conversion.
    /// Implementations that do not perform any conversion
    /// always return zero clipped samples.
    ///
    /// If an invalid channel number is given,
    /// or if either of the channels is to short to copy `take` values,
    /// no values will be copied and `None` is returned.
    fn copy_from_other_to_channel(
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
    /// Copies `count` frames, from the range `src..src+count`,
    /// to the range `dest..dest+count`.
    /// The two regions are allowed to overlap.
    /// The default implementation copies by calling the read and write methods,
    /// while type specific implementations can use more efficient methods.
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

    /// Copy a single sample within the buffer.
    /// Copies from the source to the target frame and channel.
    /// Returns `true` if the sample was copied, and `false` if not,
    /// which may happen if the source or target frame or channel was out of bounds.
    /// The default implementation copies by calling the read and write methods,
    /// while type specific implementations can use more efficient methods.
    fn copy_sample_within(
        &mut self,
        source_channel: usize,
        source_frame: usize,
        target_channel: usize,
        target_frame: usize,
    ) -> bool {
        if let Some(value) = self.read_sample(source_channel, source_frame) {
            if self
                .write_sample(target_channel, target_frame, &value)
                .is_some()
            {
                return true;
            }
        }
        false
    }

    /// Swap two samples in the buffer.
    /// Returns `true` if the samples were swapped, and `false` if not,
    /// which may happen if the channel or frame for either sample was out of bounds.
    /// The default implementation copies by calling the read and write methods,
    /// while type specific implementations can use more efficient methods.
    fn swap_samples(
        &mut self,
        channel_a: usize,
        frame_a: usize,
        channel_b: usize,
        frame_b: usize,
    ) -> bool {
        if let Some(value_a) = self.read_sample(channel_a, frame_a) {
            if let Some(value_b) = self.read_sample(channel_b, frame_b) {
                // both values could be read, thus both frame&channel combinations are valid
                unsafe {
                    self.write_sample_unchecked(channel_b, frame_b, &value_a);
                    self.write_sample_unchecked(channel_a, frame_a, &value_b);
                    return true;
                }
            }
        }
        false
    }
}

//   _____         _
//  |_   _|__  ___| |_ ___
//    | |/ _ \/ __| __/ __|
//    | |  __/\__ \ |_\__ \
//    |_|\___||___/\__|___/

#[cfg(test)]
mod tests {
    extern crate alloc;

    use crate::tests::MinimalAdapter;
    use crate::{Adapter, AdapterMut};
    use alloc::vec;

    fn dummy_adapter() -> MinimalAdapter<i32> {
        let data = vec![1_i32, 1, 2, 3, 4, 5, 6, 7];
        MinimalAdapter::new_from_vec(data, 2, 4)
    }

    #[test]
    fn read_sample() {
        let buffer = dummy_adapter();
        assert_eq!(buffer.read_sample(0, 0), Some(1));
        assert_eq!(buffer.read_sample(1, 3), Some(7));
        assert_eq!(buffer.read_sample(2, 0), None); // OOB channel
        assert_eq!(buffer.read_sample(0, 4), None); // OOB frame
    }

    #[test]
    fn copy_from_channel_to_slice() {
        let buffer = dummy_adapter();
        let mut slice = [0; 3];
        // ch 0 is [1, 2, 4, 6]
        let copied = buffer.copy_from_channel_to_slice(0, 1, &mut slice);
        assert_eq!(copied, 3);
        assert_eq!(slice, [2, 4, 6]);

        // OOB channel
        let mut slice2 = [0; 2];
        let copied = buffer.copy_from_channel_to_slice(2, 0, &mut slice2);
        assert_eq!(copied, 0);
        assert_eq!(slice2, [0, 0]);

        // OOB skip
        let copied = buffer.copy_from_channel_to_slice(0, 4, &mut slice2);
        assert_eq!(copied, 0);

        // Slice larger than available frames
        let mut slice3 = [0; 5];
        let copied = buffer.copy_from_channel_to_slice(0, 0, &mut slice3);
        assert_eq!(copied, 4);
        assert_eq!(slice3, [1, 2, 4, 6, 0]);
    }

    #[test]
    fn copy_from_frame_to_slice() {
        let buffer = dummy_adapter();
        let mut slice = [0; 1];
        // frame 1 is [2, 3]
        let copied = buffer.copy_from_frame_to_slice(1, 1, &mut slice);
        assert_eq!(copied, 1);
        assert_eq!(slice, [3]);

        // OOB frame
        let mut slice2 = [0; 2];
        let copied = buffer.copy_from_frame_to_slice(4, 0, &mut slice2);
        assert_eq!(copied, 0);
        assert_eq!(slice2, [0, 0]);

        // OOB skip
        let copied = buffer.copy_from_frame_to_slice(0, 2, &mut slice2);
        assert_eq!(copied, 0);

        // Slice larger than available channels
        let mut slice3 = [0; 3];
        let copied = buffer.copy_from_frame_to_slice(1, 0, &mut slice3);
        assert_eq!(copied, 2);
        assert_eq!(slice3, [2, 3, 0]);
    }

    #[test]
    fn write_sample() {
        let mut buffer = dummy_adapter();
        assert_eq!(buffer.write_sample(0, 0, &100), Some(false));
        assert_eq!(buffer.read_sample(0, 0), Some(100));
        assert_eq!(buffer.write_sample(2, 0, &101), None); // OOB channel
        assert_eq!(buffer.write_sample(0, 4, &102), None); // OOB frame
    }

    #[test]
    fn copy_from_slice_to_channel() {
        let mut buffer = dummy_adapter();
        let slice = [10, 11];
        let (copied, clipped) = buffer.copy_from_slice_to_channel(0, 1, &slice);
        assert_eq!(copied, 2);
        assert_eq!(clipped, 0);
        // ch 0 was [1, 2, 4, 6], now [1, 10, 11, 6]
        assert_eq!(buffer.read_sample(0, 0), Some(1));
        assert_eq!(buffer.read_sample(0, 1), Some(10));
        assert_eq!(buffer.read_sample(0, 2), Some(11));
        assert_eq!(buffer.read_sample(0, 3), Some(6));
    }

    #[test]
    fn copy_from_slice_to_frame() {
        let mut buffer = dummy_adapter();
        let slice = [10];
        let (copied, clipped) = buffer.copy_from_slice_to_frame(1, 1, &slice);
        assert_eq!(copied, 1);
        assert_eq!(clipped, 0);
        // frame 1 was [2, 3], now [2, 10]
        assert_eq!(buffer.read_sample(0, 1), Some(2));
        assert_eq!(buffer.read_sample(1, 1), Some(10));
    }

    #[test]
    fn copy_from_other_to_channel() {
        let mut buffer = dummy_adapter();
        let other = dummy_adapter();
        // copy ch1 from other to ch0 in buffer
        // other ch1: [1, 3, 5, 7]
        let clipped = buffer.copy_from_other_to_channel(&other, 1, 0, 0, 0, 4);
        assert_eq!(clipped, Some(0));
        assert_eq!(buffer.read_sample(0, 0), Some(1));
        assert_eq!(buffer.read_sample(0, 1), Some(3));
        assert_eq!(buffer.read_sample(0, 2), Some(5));
        assert_eq!(buffer.read_sample(0, 3), Some(7));
    }

    #[test]
    fn fill_channel_with() {
        let mut buffer = dummy_adapter();
        buffer.fill_channel_with(0, &9).unwrap();
        assert_eq!(buffer.read_sample(0, 0), Some(9));
        assert_eq!(buffer.read_sample(0, 1), Some(9));
        assert_eq!(buffer.read_sample(0, 2), Some(9));
        assert_eq!(buffer.read_sample(0, 3), Some(9));
        assert_eq!(buffer.read_sample(1, 0), Some(1)); // other channel unaffected
    }

    #[test]
    fn fill_frame_with() {
        let mut buffer = dummy_adapter();
        buffer.fill_frame_with(1, &9).unwrap();
        assert_eq!(buffer.read_sample(0, 1), Some(9));
        assert_eq!(buffer.read_sample(1, 1), Some(9));
        assert_eq!(buffer.read_sample(0, 0), Some(1)); // other frame unaffected
    }

    #[test]
    fn fill_frames_with() {
        let mut buffer = dummy_adapter();
        buffer.fill_frames_with(1, 2, &9).unwrap();
        assert_eq!(buffer.read_sample(0, 0), Some(1));
        assert_eq!(buffer.read_sample(0, 1), Some(9));
        assert_eq!(buffer.read_sample(0, 2), Some(9));
        assert_eq!(buffer.read_sample(0, 3), Some(6));
    }

    #[test]
    fn fill_with() {
        let mut buffer = dummy_adapter();
        buffer.fill_with(&9);
        for f in 0..buffer.frames() {
            for c in 0..buffer.channels() {
                assert_eq!(buffer.read_sample(c, f), Some(9));
            }
        }
    }

    #[test]
    fn copy_frames_within_forward_overlap() {
        let mut buffer = dummy_adapter();
        // copy 2 frames from frame 0 to frame 1.
        // Before: F0:[1,1], F1:[2,3], F2:[4,5], F3:[6,7]
        // After:  F0:[1,1], F1:[1,1], F2:[2,3], F3:[6,7]
        let copied = buffer.copy_frames_within(0, 1, 2).unwrap();
        assert_eq!(copied, 2);
        assert_eq!(buffer.read_sample(0, 0), Some(1));
        assert_eq!(buffer.read_sample(1, 0), Some(1));
        assert_eq!(buffer.read_sample(0, 1), Some(1));
        assert_eq!(buffer.read_sample(1, 1), Some(1));
        assert_eq!(buffer.read_sample(0, 2), Some(2));
        assert_eq!(buffer.read_sample(1, 2), Some(3));
        assert_eq!(buffer.read_sample(0, 3), Some(6));
        assert_eq!(buffer.read_sample(1, 3), Some(7));
    }

    #[test]
    fn copy_frames_within_backward_overlap() {
        let mut buffer = dummy_adapter();
        // copy 2 frames from frame 1 to frame 0.
        // Before: F0:[1,1], F1:[2,3], F2:[4,5], F3:[6,7]
        // After:  F0:[2,3], F1:[4,5], F2:[4,5], F3:[6,7]
        let copied = buffer.copy_frames_within(1, 0, 2).unwrap();
        assert_eq!(copied, 2);
        assert_eq!(buffer.read_sample(0, 0), Some(2));
        assert_eq!(buffer.read_sample(1, 0), Some(3));
        assert_eq!(buffer.read_sample(0, 1), Some(4));
        assert_eq!(buffer.read_sample(1, 1), Some(5));
        assert_eq!(buffer.read_sample(0, 2), Some(4));
        assert_eq!(buffer.read_sample(1, 2), Some(5));
        assert_eq!(buffer.read_sample(0, 3), Some(6));
        assert_eq!(buffer.read_sample(1, 3), Some(7));
    }

    #[test]
    fn copy_sample_within() {
        let mut buffer = dummy_adapter();
        // Before: (0,0) is 1, (1,1) is 3
        let success = buffer.copy_sample_within(0, 0, 1, 1);
        // After: (0,0) is 1, (1,1) is 1
        assert!(success);
        assert_eq!(buffer.read_sample(0, 0), Some(1));
        assert_eq!(buffer.read_sample(1, 1), Some(1));

        // OOB source
        assert!(!buffer.copy_sample_within(2, 0, 0, 0));
        // OOB target
        assert!(!buffer.copy_sample_within(0, 0, 2, 0));
    }

    #[test]
    fn swap_samples() {
        let mut buffer = dummy_adapter();
        // Before: (0,0) is 1, (1,1) is 3
        let success = buffer.swap_samples(0, 0, 1, 1);
        // After: (0,0) is 3, (1,1) is 1
        assert!(success);
        assert_eq!(buffer.read_sample(0, 0), Some(3));
        assert_eq!(buffer.read_sample(1, 1), Some(1));

        // OOB
        assert!(!buffer.swap_samples(0, 0, 2, 0));
    }
}
