//! # [symphonia](https://crates.io/crates/symphonia) crate compatibility
//!
//! This module implements the `audioadapter` traits for the planar
//! `AudioBuffer<S>` type from [symphonia](https://crates.io/crates/symphonia)
//! (via `symphonia-core`).
//!
//! A `symphonia` `AudioBuffer` stores its samples planar, one contiguous plane
//! per channel, so channel access is a plain slice and copying is fast.
//!
//! Wrap a shared reference for read-only access, or a mutable reference for
//! read-write access:
//!
//! ```
//! # use symphonia_core::audio::{AudioBuffer, AudioSpec, Channels};
//! use audioadapter::Adapter;
//! use audioadapter_compat_symphonia::SymphoniaAdapter;
//!
//! # let spec = AudioSpec::new(44100, Channels::Discrete(2));
//! # let mut buf: AudioBuffer<f32> = AudioBuffer::new(spec, 4);
//! # buf.render_silence(Some(4));
//! let adapter = SymphoniaAdapter::new(&buf);
//! assert_eq!(adapter.channels(), 2);
//! assert_eq!(adapter.frames(), 4);
//! ```

use audioadapter::{Adapter, AdapterMut};

use symphonia_core::audio::sample::Sample;
use symphonia_core::audio::{Audio, AudioBuffer, AudioMut};

/// A wrapper implementing the `audioadapter` traits for a `symphonia`
/// [`AudioBuffer`].
///
/// Construct it from either a `&AudioBuffer<S>` for read-only access, or a
/// `&mut AudioBuffer<S>` for read-write access.
pub struct SymphoniaAdapter<U> {
    buf: U,
}

impl<'a, S> SymphoniaAdapter<&'a AudioBuffer<S>>
where
    S: Sample,
{
    /// Create a read-only adapter wrapping a shared reference to an `AudioBuffer`.
    pub fn new(buf: &'a AudioBuffer<S>) -> Self {
        Self { buf }
    }
}

impl<'a, S> SymphoniaAdapter<&'a mut AudioBuffer<S>>
where
    S: Sample,
{
    /// Create a read-write adapter wrapping a mutable reference to an `AudioBuffer`.
    pub fn new_mut(buf: &'a mut AudioBuffer<S>) -> Self {
        Self { buf }
    }
}

#[inline]
fn spec_channels<S: Sample>(buf: &AudioBuffer<S>) -> usize {
    buf.spec().channels().count()
}

unsafe impl<S> Adapter<S> for SymphoniaAdapter<&AudioBuffer<S>>
where
    S: Sample,
{
    fn channels(&self) -> usize {
        spec_channels(self.buf)
    }

    fn frames(&self) -> usize {
        self.buf.frames()
    }

    unsafe fn read_sample_unchecked(&self, channel: usize, frame: usize) -> S {
        *unsafe { self.buf.plane(channel).unwrap().get_unchecked(frame) }
    }

    fn copy_from_channel_to_slice(&self, channel: usize, skip: usize, slice: &mut [S]) -> usize {
        if channel >= self.channels() || skip >= self.frames() {
            return 0;
        }
        let plane = self.buf.plane(channel).unwrap();
        let available = plane.len() - skip;
        let to_copy = available.min(slice.len());
        slice[..to_copy].copy_from_slice(&plane[skip..skip + to_copy]);
        to_copy
    }
}

unsafe impl<S> Adapter<S> for SymphoniaAdapter<&mut AudioBuffer<S>>
where
    S: Sample,
{
    fn channels(&self) -> usize {
        spec_channels(self.buf)
    }

    fn frames(&self) -> usize {
        self.buf.frames()
    }

    unsafe fn read_sample_unchecked(&self, channel: usize, frame: usize) -> S {
        *unsafe { self.buf.plane(channel).unwrap().get_unchecked(frame) }
    }

    fn copy_from_channel_to_slice(&self, channel: usize, skip: usize, slice: &mut [S]) -> usize {
        if channel >= self.channels() || skip >= self.frames() {
            return 0;
        }
        let plane = self.buf.plane(channel).unwrap();
        let available = plane.len() - skip;
        let to_copy = available.min(slice.len());
        slice[..to_copy].copy_from_slice(&plane[skip..skip + to_copy]);
        to_copy
    }
}

unsafe impl<S> AdapterMut<S> for SymphoniaAdapter<&mut AudioBuffer<S>>
where
    S: Sample,
{
    unsafe fn write_sample_unchecked(&mut self, channel: usize, frame: usize, value: &S) -> bool {
        unsafe {
            *self
                .buf
                .plane_mut(channel)
                .unwrap()
                .get_unchecked_mut(frame) = *value
        };
        false
    }

    fn copy_from_slice_to_channel(
        &mut self,
        channel: usize,
        skip: usize,
        slice: &[S],
    ) -> (usize, usize) {
        if channel >= Adapter::channels(self) || skip >= Adapter::frames(self) {
            return (0, 0);
        }
        let plane = self.buf.plane_mut(channel).unwrap();
        let available = plane.len() - skip;
        let to_copy = available.min(slice.len());
        plane[skip..skip + to_copy].copy_from_slice(&slice[..to_copy]);
        (to_copy, 0)
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
    use symphonia_core::audio::{AudioMut, AudioSpec, Channels};

    fn make_buffer() -> AudioBuffer<f32> {
        let spec = AudioSpec::new(44100, Channels::Discrete(2));
        let mut buf = AudioBuffer::<f32>::new(spec, 4);
        buf.render_silence(Some(4));
        // channel 0: 0,1,2,3   channel 1: 10,11,12,13
        for (frame, value) in buf.plane_mut(0).unwrap().iter_mut().enumerate() {
            *value = frame as f32;
        }
        for (frame, value) in buf.plane_mut(1).unwrap().iter_mut().enumerate() {
            *value = 10.0 + frame as f32;
        }
        buf
    }

    #[test]
    fn dimensions() {
        let buf = make_buffer();
        let adapter = SymphoniaAdapter::new(&buf);
        assert_eq!(adapter.channels(), 2);
        assert_eq!(adapter.frames(), 4);
    }

    #[test]
    fn read() {
        let buf = make_buffer();
        let adapter = SymphoniaAdapter::new(&buf);
        assert_eq!(adapter.read_sample(0, 0), Some(0.0));
        assert_eq!(adapter.read_sample(0, 3), Some(3.0));
        assert_eq!(adapter.read_sample(1, 0), Some(10.0));
        assert_eq!(adapter.read_sample(1, 3), Some(13.0));
        assert_eq!(adapter.read_sample(2, 0), None);
        assert_eq!(adapter.read_sample(0, 4), None);
    }

    #[test]
    fn copy_channel_to_slice() {
        let buf = make_buffer();
        let adapter = SymphoniaAdapter::new(&buf);
        let mut out = [0.0f32; 3];
        let copied = adapter.copy_from_channel_to_slice(1, 1, &mut out);
        assert_eq!(copied, 3);
        assert_eq!(out, [11.0, 12.0, 13.0]);
    }

    #[test]
    fn write() {
        let mut buf = make_buffer();
        let mut adapter = SymphoniaAdapter::new_mut(&mut buf);
        assert_eq!(adapter.write_sample(0, 0, &99.0), Some(false));
        assert_eq!(adapter.read_sample(0, 0), Some(99.0));
        assert_eq!(adapter.write_sample(2, 0, &1.0), None);
    }

    #[test]
    fn copy_slice_to_channel() {
        let mut buf = make_buffer();
        let mut adapter = SymphoniaAdapter::new_mut(&mut buf);
        let src = [7.0f32, 8.0];
        let (copied, clipped) = adapter.copy_from_slice_to_channel(0, 1, &src);
        assert_eq!((copied, clipped), (2, 0));
        assert_eq!(adapter.read_sample(0, 0), Some(0.0));
        assert_eq!(adapter.read_sample(0, 1), Some(7.0));
        assert_eq!(adapter.read_sample(0, 2), Some(8.0));
        assert_eq!(adapter.read_sample(0, 3), Some(3.0));
    }
}
