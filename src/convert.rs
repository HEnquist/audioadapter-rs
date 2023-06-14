use crate::{BufferSizeError, ChannelSamples, FrameSamples, Frames, Channels, implement_size_getters, implement_iterators};
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

/// Wrapper for a slice of length `frames * channels`.
/// The samples are stored in _interleaved_ order,
/// where all the samples for one frame are stored consecutively,
/// followed by the samples for the next frame.
/// For a stereo buffer containing four frames, the order is
/// `L1, R1, L2, R2, L3, R3, L4, R4`
pub struct ConvertingInterleavedS32<U, V> {
    _phantom: core::marker::PhantomData<V>,
    buf: U,
    frames: usize,
    channels: usize,
    bytes_per_sample: usize,
}

impl<U, V> ConvertingInterleavedS32<U, V> {
    fn calc_index(&self, channel: usize, frame: usize) -> usize {
        self.bytes_per_sample * (frame * self.channels + channel)
    }
}

impl<'a, T> ConvertingInterleavedS32<&'a [u8], T> where T: 'a {
    /// Create a new `InterleavedWrapper` to wrap a slice.
    /// The slice length must be at least `frames*channels`.
    /// It is allowed to be longer than needed,
    /// but these extra values cannot
    /// be accessed via the `AudioBuffer` trait methods.
    pub fn new(buf: &'a [u8], channels: usize, frames: usize) -> Result<Self, BufferSizeError> {
        if buf.len() < frames * channels {
            return Err(BufferSizeError {
                desc: format!("Buffer is too short, {} < {}", buf.len(), frames * channels),
            });
        }
        Ok(Self {
            _phantom: core::marker::PhantomData,
            buf,
            frames,
            channels,
            bytes_per_sample: 4,
        })
    }
}
/*
impl<'a, T> InterleavedSlice<&'a mut [T]> {
    /// Create a new `InterleavedWrapper` to wrap a mutable slice.
    /// The slice length must be at least `frames*channels`.
    /// It is allowed to be longer than needed,
    /// but these extra values cannot
    /// be accessed via the `AudioBuffer` trait methods.
    pub fn new_mut(
        buf: &'a mut [T],
        channels: usize,
        frames: usize,
    ) -> Result<Self, BufferSizeError> {
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
 */


impl<'a, T> ConvertingAudioBuffer<'a, T> for ConvertingInterleavedS32<&'a [u8], T>
where
    T: Sample<T> + 'a,
{
    unsafe fn read_unchecked(&self, channel: usize, frame: usize) -> T {
        let index = self.calc_index(channel, frame);
        T::from_s32_le(self.buf[index..index+self.bytes_per_sample].try_into().unwrap())
    }

    implement_size_getters!();
}
/*
impl<'a, T> AudioBuffer<'a, T> for InterleavedSlice<&'a mut [T]>
where
    T: Clone,
{
    unsafe fn get_unchecked(&self, channel: usize, frame: usize) -> &T {
        let index = self.calc_index(channel, frame);
        return self.buf.get_unchecked(index);
    }

    implement_size_getters!();

    implement_iterators!();

    fn write_from_frame_to_slice(&self, frame: usize, start: usize, slice: &mut [T]) -> usize {
        if frame >= self.frames || start >= self.channels {
            return 0;
        }
        let channels_to_write = if (self.channels - start) < slice.len() {
            self.channels - start
        } else {
            slice.len()
        };
        let buffer_start = self.calc_index(start, frame);
        slice[..channels_to_write]
            .clone_from_slice(&self.buf[buffer_start..buffer_start + channels_to_write]);
        channels_to_write
    }
}

impl<'a, T> AudioBufferMut<'a, T> for InterleavedSlice<&'a mut [T]>
where
    T: Clone,
{
    unsafe fn get_unchecked_mut(&mut self, channel: usize, frame: usize) -> &mut T {
        let index = self.calc_index(channel, frame);
        return self.buf.get_unchecked_mut(index);
    }

    implement_iterators_mut!();

    fn read_into_frame_from_slice(&mut self, frame: usize, start: usize, slice: &[T]) -> usize {
        if frame >= self.frames || start >= self.channels {
            return 0;
        }
        let channels_to_read = if (self.channels - start) < slice.len() {
            self.channels - start
        } else {
            slice.len()
        };
        let buffer_start = self.calc_index(start, frame);
        self.buf[buffer_start..buffer_start + channels_to_read]
            .clone_from_slice(&slice[..channels_to_read]);
        channels_to_read
    }
}
*/


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
            0, 0, 0, 0,
            0, 0, 0, 128,
            0, 0, 0, 64,
            0, 0, 0, 192,
            0, 0, 0, 32,
            0, 0, 0, 224,
        ];
        let buffer: ConvertingInterleavedS32<&[u8], f32> = ConvertingInterleavedS32::new(&data, 2, 3).unwrap();
        assert_eq!(buffer.read(0,0).unwrap(), 0.0);
        assert_eq!(buffer.read(1,0).unwrap(), -1.0);
        assert_eq!(buffer.read(0,1).unwrap(), 0.5);
        assert_eq!(buffer.read(1,1).unwrap(), -0.5);
        assert_eq!(buffer.read(0,2).unwrap(), 0.25);
        assert_eq!(buffer.read(1,2).unwrap(), -0.25);
    }
}