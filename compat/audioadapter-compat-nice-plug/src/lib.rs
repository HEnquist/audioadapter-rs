//! # [nice-plug](https://codeberg.org/RustAudio/nice-plug) compatibility
//!
//! This crate implements the [audioadapter](https://crates.io/crates/audioadapter)
//! traits for the `Buffer` type from the `nice-plug` VST3/CLAP plugin framework,
//! via its crates.io-published core crate
//! [nice-plug-core](https://crates.io/crates/nice-plug-core).
//!
//! `nice-plug` is a maintained continuation of
//! [nih-plug](https://github.com/robbert-vdh/nih-plug) and shares its buffer
//! layout: planar, non-interleaved, one `&mut [f32]` slice per channel. This
//! makes it a drop-in way to feed `audioadapter`-based processing directly from
//! a plugin's audio buffer.
//!
//! Wrap a shared reference for read-only access, or a mutable reference for
//! read-write access:
//!
//! ```
//! use audioadapter::Adapter;
//! use audioadapter_compat_nice_plug::NicePlugAdapter;
//! use nice_plug_core::buffer::Buffer;
//!
//! // In a real plugin the host hands you a `&mut Buffer` in `process()`.
//! let mut channels: Vec<Vec<f32>> = vec![vec![0.0, 1.0], vec![2.0, 3.0]];
//! let mut buffer = Buffer::default();
//! unsafe {
//!     buffer.set_slices(2, |slices| {
//!         let (left, right) = channels.split_at_mut(1);
//!         *slices = vec![left[0].as_mut_slice(), right[0].as_mut_slice()];
//!     });
//! }
//!
//! let adapter = NicePlugAdapter::new(&buffer);
//! assert_eq!(adapter.channels(), 2);
//! assert_eq!(adapter.frames(), 2);
//! assert_eq!(adapter.read_sample(1, 0), Some(2.0));
//! ```

use audioadapter::{Adapter, AdapterMut};
use nice_plug_core::buffer::Buffer;

/// A wrapper implementing the `audioadapter` traits for a `nice-plug`
/// [`Buffer`].
///
/// Construct it from either a `&Buffer` for read-only access, or a `&mut Buffer`
/// for read-write access. The sample type is always `f32`.
pub struct NicePlugAdapter<U> {
    buf: U,
}

impl<'b, 'a> NicePlugAdapter<&'b Buffer<'a>> {
    /// Create a read-only adapter wrapping a shared reference to a `Buffer`.
    pub fn new(buf: &'b Buffer<'a>) -> Self {
        Self { buf }
    }
}

impl<'b, 'a> NicePlugAdapter<&'b mut Buffer<'a>> {
    /// Create a read-write adapter wrapping a mutable reference to a `Buffer`.
    pub fn new_mut(buf: &'b mut Buffer<'a>) -> Self {
        Self { buf }
    }
}

unsafe impl Adapter<f32> for NicePlugAdapter<&Buffer<'_>> {
    fn channels(&self) -> usize {
        self.buf.channels()
    }

    fn frames(&self) -> usize {
        self.buf.samples()
    }

    unsafe fn read_sample_unchecked(&self, channel: usize, frame: usize) -> f32 {
        let planes = self.buf.as_slice_immutable();
        unsafe { *planes.get_unchecked(channel).get_unchecked(frame) }
    }

    fn copy_from_channel_to_slice(&self, channel: usize, skip: usize, slice: &mut [f32]) -> usize {
        if channel >= self.channels() || skip >= self.frames() {
            return 0;
        }
        let plane: &[f32] = self.buf.as_slice_immutable()[channel];
        let available = plane.len() - skip;
        let to_copy = available.min(slice.len());
        slice[..to_copy].copy_from_slice(&plane[skip..skip + to_copy]);
        to_copy
    }
}

unsafe impl Adapter<f32> for NicePlugAdapter<&mut Buffer<'_>> {
    fn channels(&self) -> usize {
        self.buf.channels()
    }

    fn frames(&self) -> usize {
        self.buf.samples()
    }

    unsafe fn read_sample_unchecked(&self, channel: usize, frame: usize) -> f32 {
        let planes = self.buf.as_slice_immutable();
        unsafe { *planes.get_unchecked(channel).get_unchecked(frame) }
    }

    fn copy_from_channel_to_slice(&self, channel: usize, skip: usize, slice: &mut [f32]) -> usize {
        if channel >= self.channels() || skip >= self.frames() {
            return 0;
        }
        let plane: &[f32] = self.buf.as_slice_immutable()[channel];
        let available = plane.len() - skip;
        let to_copy = available.min(slice.len());
        slice[..to_copy].copy_from_slice(&plane[skip..skip + to_copy]);
        to_copy
    }
}

unsafe impl AdapterMut<f32> for NicePlugAdapter<&mut Buffer<'_>> {
    unsafe fn write_sample_unchecked(&mut self, channel: usize, frame: usize, value: &f32) -> bool {
        let planes = self.buf.as_slice();
        unsafe { *planes.get_unchecked_mut(channel).get_unchecked_mut(frame) = *value };
        false
    }

    fn copy_from_slice_to_channel(
        &mut self,
        channel: usize,
        skip: usize,
        slice: &[f32],
    ) -> (usize, usize) {
        if channel >= Adapter::channels(self) || skip >= Adapter::frames(self) {
            return (0, 0);
        }
        let plane: &mut [f32] = self.buf.as_slice()[channel];
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

    fn with_buffer<R>(channels: &mut [Vec<f32>], f: impl FnOnce(&mut Buffer) -> R) -> R {
        let num_samples = channels[0].len();
        let mut slices: Vec<&mut [f32]> = channels.iter_mut().map(|c| c.as_mut_slice()).collect();
        let mut buffer = Buffer::default();
        unsafe {
            buffer.set_slices(num_samples, |output| {
                *output = core::mem::take(&mut slices);
            });
        }
        f(&mut buffer)
    }

    #[test]
    fn dimensions_and_read() {
        let mut channels = vec![vec![0.0, 1.0, 2.0, 3.0], vec![10.0, 11.0, 12.0, 13.0]];
        with_buffer(&mut channels, |buffer| {
            let adapter = NicePlugAdapter::new(buffer);
            assert_eq!(adapter.channels(), 2);
            assert_eq!(adapter.frames(), 4);
            assert_eq!(adapter.read_sample(0, 0), Some(0.0));
            assert_eq!(adapter.read_sample(1, 3), Some(13.0));
            assert_eq!(adapter.read_sample(2, 0), None);
            assert_eq!(adapter.read_sample(0, 4), None);
        });
    }

    #[test]
    fn copy_channel_to_slice() {
        let mut channels = vec![vec![0.0, 1.0, 2.0, 3.0], vec![10.0, 11.0, 12.0, 13.0]];
        with_buffer(&mut channels, |buffer| {
            let adapter = NicePlugAdapter::new(buffer);
            let mut out = [0.0f32; 3];
            let copied = adapter.copy_from_channel_to_slice(1, 1, &mut out);
            assert_eq!(copied, 3);
            assert_eq!(out, [11.0, 12.0, 13.0]);
        });
    }

    #[test]
    fn write_and_copy_from_slice() {
        let mut channels = vec![vec![0.0, 0.0, 0.0, 0.0], vec![0.0, 0.0, 0.0, 0.0]];
        with_buffer(&mut channels, |buffer| {
            let mut adapter = NicePlugAdapter::new_mut(buffer);
            assert_eq!(adapter.write_sample(0, 0, &9.0), Some(false));
            assert_eq!(adapter.read_sample(0, 0), Some(9.0));
            let (copied, clipped) = adapter.copy_from_slice_to_channel(1, 1, &[7.0, 8.0]);
            assert_eq!((copied, clipped), (2, 0));
            assert_eq!(adapter.read_sample(1, 1), Some(7.0));
            assert_eq!(adapter.read_sample(1, 2), Some(8.0));
            assert_eq!(adapter.write_sample(2, 0, &1.0), None);
        });
    }
}
