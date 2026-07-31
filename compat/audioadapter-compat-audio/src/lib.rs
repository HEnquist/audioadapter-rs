//! # [audio](https://crates.io/crates/audio) crate compatibility
//!
//! This module implements the `audioadapter` traits for buffers from the
//! [audio](https://crates.io/crates/audio) crate.
//!
//! The buffer is wrapped in an [`AudioBufAdapter`], which implements
//! [`Adapter`] for any `audio` buffer that implements `Buf` +
//! `ExactSizeBuf`, and [`AdapterMut`] when it also implements
//! `BufMut`.
//!
//! ```
//! use audio::wrap;
//! use audioadapter::Adapter;
//! use audioadapter_compat_audio::AudioBufAdapter;
//!
//! let buf = wrap::interleaved(&[1, 2, 3, 4, 5, 6, 7, 8], 2);
//! let adapter = AudioBufAdapter::new(buf);
//! assert_eq!(adapter.read_sample(0, 0), Some(1));
//! assert_eq!(adapter.read_sample(1, 0), Some(2));
//! ```

use audioadapter::{Adapter, AdapterMut};

use audio_core::{Buf, BufMut, Channel, ChannelMut, ExactSizeBuf, Sample};

/// A wrapper implementing the `audioadapter` traits for a buffer
/// from the [audio](https://crates.io/crates/audio) crate.
///
/// The wrapped value may own its data or borrow it, for example
/// `audio::buf::Interleaved<T>` or the result of `audio::wrap::interleaved(..)`.
pub struct AudioBufAdapter<U> {
    buf: U,
}

impl<U> AudioBufAdapter<U> {
    /// Create a new adapter wrapping an `audio` crate buffer.
    pub fn new(buf: U) -> Self {
        Self { buf }
    }

    /// Consume the adapter and return the wrapped buffer.
    pub fn into_inner(self) -> U {
        self.buf
    }

    /// Get a reference to the wrapped buffer.
    pub fn inner(&self) -> &U {
        &self.buf
    }

    /// Get a mutable reference to the wrapped buffer.
    pub fn inner_mut(&mut self) -> &mut U {
        &mut self.buf
    }
}

unsafe impl<T, U> Adapter<T> for AudioBufAdapter<U>
where
    T: Clone + Sample,
    U: Buf<Sample = T> + ExactSizeBuf<Sample = T>,
{
    fn channels(&self) -> usize {
        self.buf.channels()
    }

    fn frames(&self) -> usize {
        self.buf.frames()
    }

    unsafe fn read_sample_unchecked(&self, channel: usize, frame: usize) -> T {
        self.buf.get_channel(channel).unwrap().get(frame).unwrap()
    }

    fn copy_from_channel_to_slice(&self, channel: usize, skip: usize, slice: &mut [T]) -> usize {
        if channel >= self.channels() || skip >= self.frames() {
            return 0;
        }
        let frames_to_write = if (self.frames() - skip) < slice.len() {
            self.frames() - skip
        } else {
            slice.len()
        };
        let chan = self.buf.get_channel(channel).unwrap();
        chan.iter()
            .skip(skip)
            .take(frames_to_write)
            .zip(slice.iter_mut())
            .for_each(|(s, o)| *o = s);
        frames_to_write
    }
}

unsafe impl<T, U> AdapterMut<T> for AudioBufAdapter<U>
where
    T: Clone + Sample,
    U: BufMut<Sample = T> + ExactSizeBuf<Sample = T>,
{
    unsafe fn write_sample_unchecked(&mut self, channel: usize, frame: usize, value: &T) -> bool {
        *self
            .buf
            .get_channel_mut(channel)
            .unwrap()
            .get_mut(frame)
            .unwrap() = *value;
        false
    }

    fn copy_from_slice_to_channel(
        &mut self,
        channel: usize,
        skip: usize,
        slice: &[T],
    ) -> (usize, usize) {
        if channel >= Adapter::channels(self) || skip >= Adapter::frames(self) {
            return (0, 0);
        }
        let frames_to_read = if (Adapter::frames(self) - skip) < slice.len() {
            Adapter::frames(self) - skip
        } else {
            slice.len()
        };
        let mut chan = self.buf.get_channel_mut(channel).unwrap();
        chan.iter_mut()
            .skip(skip)
            .take(frames_to_read)
            .zip(slice.iter())
            .for_each(|(s, o)| *s = *o);
        (frames_to_read, 0)
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
    use audio::wrap;

    #[test]
    fn read_indirect() {
        let buf = AudioBufAdapter::new(wrap::interleaved(&[1, 2, 3, 4, 5, 6, 7, 8], 2));
        assert_eq!(unsafe { buf.read_sample_unchecked(0, 0) }, 1);
        assert_eq!(unsafe { buf.read_sample_unchecked(1, 0) }, 2);
        assert_eq!(unsafe { buf.read_sample_unchecked(0, 1) }, 3);
        assert_eq!(unsafe { buf.read_sample_unchecked(1, 1) }, 4);
    }

    #[test]
    fn write_indirect() {
        let mut buf = AudioBufAdapter::new(audio::buf::Interleaved::<i32>::with_topology(2, 4));
        unsafe {
            buf.write_sample_unchecked(0, 0, &1);
            buf.write_sample_unchecked(1, 0, &2);
            buf.write_sample_unchecked(0, 1, &3);
            buf.write_sample_unchecked(1, 1, &4);
        }
        let inner = buf.inner();
        assert_eq!(inner.get_channel(0).unwrap().get(0).unwrap(), 1);
        assert_eq!(inner.get_channel(1).unwrap().get(0).unwrap(), 2);
        assert_eq!(inner.get_channel(0).unwrap().get(1).unwrap(), 3);
        assert_eq!(inner.get_channel(1).unwrap().get(1).unwrap(), 4);
    }

    #[test]
    fn copy_to_slice() {
        let mut other = [0; 3];
        let buf = AudioBufAdapter::new(wrap::interleaved(&[1, 2, 3, 4, 5, 6, 7, 8], 2));
        buf.copy_from_channel_to_slice(0, 1, &mut other);
        assert_eq!(other[0], 3);
        assert_eq!(other[1], 5);
        assert_eq!(other[2], 7);
    }

    #[test]
    fn copy_from_slice() {
        let other = [1, 2, 3];
        let mut buf = AudioBufAdapter::new(audio::buf::Interleaved::<i32>::with_topology(2, 4));
        buf.copy_from_slice_to_channel(0, 1, &other);
        let inner = buf.inner();
        assert_eq!(inner.get_channel(0).unwrap().get(0).unwrap(), 0);
        assert_eq!(inner.get_channel(0).unwrap().get(1).unwrap(), 1);
        assert_eq!(inner.get_channel(0).unwrap().get(2).unwrap(), 2);
        assert_eq!(inner.get_channel(0).unwrap().get(3).unwrap(), 3);
    }

    #[test]
    fn read_direct() {
        let buf = AudioBufAdapter::new(wrap::interleaved(&[1, 2, 3, 4, 5, 6, 7, 8], 2));
        assert_eq!(buf.read_sample(0, 0), Some(1));
        assert_eq!(buf.read_sample(1, 0), Some(2));
        assert_eq!(buf.read_sample(0, 1), Some(3));
        assert_eq!(buf.read_sample(1, 1), Some(4));
    }

    #[test]
    fn write_direct() {
        let mut buf = AudioBufAdapter::new(audio::buf::Interleaved::<i32>::with_topology(2, 4));
        buf.write_sample(0, 0, &1).unwrap();
        buf.write_sample(1, 0, &2).unwrap();
        buf.write_sample(0, 1, &3).unwrap();
        buf.write_sample(1, 1, &4).unwrap();
        let inner = buf.inner();
        assert_eq!(inner.get_channel(0).unwrap().get(0).unwrap(), 1);
        assert_eq!(inner.get_channel(1).unwrap().get(0).unwrap(), 2);
        assert_eq!(inner.get_channel(0).unwrap().get(1).unwrap(), 3);
        assert_eq!(inner.get_channel(1).unwrap().get(1).unwrap(), 4);
    }
}
