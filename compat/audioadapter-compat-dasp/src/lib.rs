//! # [dasp](https://crates.io/crates/dasp) crate compatibility
//!
//! This crate implements the [audioadapter](https://crates.io/crates/audioadapter)
//! traits for slices of [dasp](https://crates.io/crates/dasp) frames
//! (`&[F]` / `&mut [F]` where `F: dasp_frame::Frame`).
//!
//! In `dasp`, a multi-channel buffer is a sequence of frames, where each frame
//! holds one sample per channel (for example `[f32; 2]` for stereo). This maps
//! onto the interleaved layout: element `n` of the slice is frame `n`, and the
//! channels live within each frame.
//!
//! Wrap a shared slice for read-only access, or a mutable slice for read-write
//! access:
//!
//! ```
//! use audioadapter::Adapter;
//! use audioadapter_compat_dasp::DaspAdapter;
//!
//! let frames: [[f32; 2]; 3] = [[1.0, 2.0], [3.0, 4.0], [5.0, 6.0]];
//! let adapter = DaspAdapter::new(&frames[..]);
//! assert_eq!(adapter.channels(), 2);
//! assert_eq!(adapter.frames(), 3);
//! assert_eq!(adapter.read_sample(0, 1), Some(3.0));
//! assert_eq!(adapter.read_sample(1, 0), Some(2.0));
//! ```

use audioadapter::{Adapter, AdapterMut};
use dasp_frame::Frame;

/// A wrapper implementing the `audioadapter` traits for a slice of `dasp`
/// frames.
///
/// Construct it from either a `&[F]` for read-only access, or a `&mut [F]` for
/// read-write access.
pub struct DaspAdapter<U> {
    frames: U,
}

impl<'a, F> DaspAdapter<&'a [F]>
where
    F: Frame,
{
    /// Create a read-only adapter wrapping a slice of frames.
    pub fn new(frames: &'a [F]) -> Self {
        Self { frames }
    }
}

impl<'a, F> DaspAdapter<&'a mut [F]>
where
    F: Frame,
{
    /// Create a read-write adapter wrapping a mutable slice of frames.
    pub fn new_mut(frames: &'a mut [F]) -> Self {
        Self { frames }
    }
}

unsafe impl<F> Adapter<F::Sample> for DaspAdapter<&[F]>
where
    F: Frame,
{
    fn channels(&self) -> usize {
        F::CHANNELS
    }

    fn frames(&self) -> usize {
        self.frames.len()
    }

    unsafe fn read_sample_unchecked(&self, channel: usize, frame: usize) -> F::Sample {
        *unsafe { self.frames.get_unchecked(frame).channel_unchecked(channel) }
    }
}

unsafe impl<F> Adapter<F::Sample> for DaspAdapter<&mut [F]>
where
    F: Frame,
{
    fn channels(&self) -> usize {
        F::CHANNELS
    }

    fn frames(&self) -> usize {
        self.frames.len()
    }

    unsafe fn read_sample_unchecked(&self, channel: usize, frame: usize) -> F::Sample {
        *unsafe { self.frames.get_unchecked(frame).channel_unchecked(channel) }
    }
}

unsafe impl<F> AdapterMut<F::Sample> for DaspAdapter<&mut [F]>
where
    F: Frame,
{
    unsafe fn write_sample_unchecked(
        &mut self,
        channel: usize,
        frame: usize,
        value: &F::Sample,
    ) -> bool {
        let target = unsafe { self.frames.get_unchecked_mut(frame) };
        // `Frame` gives no per-channel mutable access, so rebuild the frame with
        // the one channel replaced. `Frame` is `Copy`, so snapshot it first.
        let old = *target;
        *target = F::from_fn(|i| {
            if i == channel {
                *value
            } else {
                *unsafe { old.channel_unchecked(i) }
            }
        });
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
    use super::*;

    #[test]
    fn dimensions_and_read() {
        let frames: [[f32; 2]; 3] = [[1.0, 2.0], [3.0, 4.0], [5.0, 6.0]];
        let adapter = DaspAdapter::new(&frames[..]);
        assert_eq!(adapter.channels(), 2);
        assert_eq!(adapter.frames(), 3);
        assert_eq!(adapter.read_sample(0, 0), Some(1.0));
        assert_eq!(adapter.read_sample(1, 0), Some(2.0));
        assert_eq!(adapter.read_sample(0, 2), Some(5.0));
        assert_eq!(adapter.read_sample(2, 0), None);
        assert_eq!(adapter.read_sample(0, 3), None);
    }

    #[test]
    fn write() {
        let mut frames: [[f32; 2]; 3] = [[1.0, 2.0], [3.0, 4.0], [5.0, 6.0]];
        let mut adapter = DaspAdapter::new_mut(&mut frames[..]);
        assert_eq!(adapter.write_sample(1, 0, &9.0), Some(false));
        assert_eq!(adapter.read_sample(1, 0), Some(9.0));
        // The other channel in the same frame is untouched.
        assert_eq!(adapter.read_sample(0, 0), Some(1.0));
        assert_eq!(adapter.write_sample(2, 0, &1.0), None);
        assert_eq!(adapter.write_sample(0, 3, &1.0), None);
    }

    #[test]
    fn copy_from_slice_to_channel_default() {
        let mut frames: [[f32; 2]; 3] = [[0.0, 0.0], [0.0, 0.0], [0.0, 0.0]];
        let mut adapter = DaspAdapter::new_mut(&mut frames[..]);
        let (copied, clipped) = adapter.copy_from_slice_to_channel(0, 0, &[1.0, 2.0, 3.0]);
        assert_eq!((copied, clipped), (3, 0));
        assert_eq!(adapter.read_sample(0, 0), Some(1.0));
        assert_eq!(adapter.read_sample(0, 2), Some(3.0));
        assert_eq!(adapter.read_sample(1, 0), Some(0.0));
    }
}
