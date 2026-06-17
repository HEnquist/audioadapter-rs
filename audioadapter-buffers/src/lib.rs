#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

/// Wrappers providing direct access to samples in buffers.
pub mod direct;
/// Wrappers providing float conversion of numeric values
/// stored both directly and as raw bytes.
pub mod number_to_float;
/// Wrappers that store their data in an owned vector.
#[cfg(feature = "alloc")]
pub mod owned;

/// Dummy Adapter
pub mod dummy;

/// Utility helpers for working with adapters, currently for copying samples
/// between an adapter and a plain interleaved or sequential slice.
pub mod utils;

/// Sample format types re-exported from [`audioadapter_sample`].
///
/// These are the byte-wrapper types (`I16_LE`, `I24_LE`, `F32_LE`, …) used with
/// the byte-based constructors and converters in [`number_to_float`] and
/// [`adapter_to_float`]. They are re-exported here so that you do not need to add
/// a separate dependency on `audioadapter-sample` just to name a format.
pub use audioadapter_sample::sample;

mod slicetools;

use core::error::Error;
use core::fmt;

pub mod adapter_to_float;

/// Error returned when the wrapped data structure has the wrong dimensions,
/// typically that it is too short.
#[derive(Debug)]
pub enum SizeError {
    /// The outer container in a sequential nested layout has too few channel buffers.
    ///
    /// This applies to structures like `&[Vec<T>]` or `&[&[T]]` used as
    /// channel-major data, where each outer element represents one channel.
    ChannelsContainer { actual: usize, required: usize },
    /// The outer container in an interleaved nested layout has too few frame buffers.
    ///
    /// This applies to structures like `&[Vec<T>]` or `&[&[T]]` used as
    /// frame-major data, where each outer element represents one frame.
    FramesContainer { actual: usize, required: usize },
    /// An inner channel buffer is too short for the requested frame count.
    ///
    /// `index` identifies which channel buffer failed the length check.
    ChannelBuffer {
        index: usize,
        actual: usize,
        required: usize,
    },
    /// An inner frame buffer is too short for the requested channel count.
    ///
    /// `index` identifies which frame buffer failed the length check.
    FrameBuffer {
        index: usize,
        actual: usize,
        required: usize,
    },
    /// A flat (non-nested) sample buffer is too short for the requested dimensions.
    ///
    /// This is used for adapters backed by a single contiguous slice/vector where
    /// the required length is computed from `channels * frames` (and possibly an
    /// additional per-sample element factor for raw byte/sample representations).
    ///
    /// `actual` is the provided flat buffer length and `required` is the minimum
    /// length needed for the requested adapter shape.
    Total { actual: usize, required: usize },
    /// A channel-activity mask has an invalid length.
    ///
    /// This applies to sparse sequential adapters where the mask must contain one
    /// boolean entry per channel.
    ///
    /// `actual` is the provided mask length and `required` is the channel count.
    Mask { actual: usize, required: usize },
}

impl Error for SizeError {}

impl fmt::Display for SizeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SizeError::ChannelsContainer { actual, required } => write!(
                f,
                "Channels container is too short, got: {}, required: {}",
                actual, required
            ),
            SizeError::FramesContainer { actual, required } => write!(
                f,
                "Frames container is too short, got: {}, required: {}",
                actual, required
            ),
            SizeError::ChannelBuffer {
                index,
                actual,
                required,
            } => write!(
                f,
                "Channel buffer {} is too short for requested frame count, got: {}, required: {}",
                index, actual, required
            ),
            SizeError::FrameBuffer {
                index,
                actual,
                required,
            } => write!(
                f,
                "Frame buffer {} is too short for requested channel count, got: {}, required: {}",
                index, actual, required
            ),
            SizeError::Total { actual, required } => write!(
                f,
                "Flat buffer is too short for requested dimensions, got: {}, required: {}",
                actual, required
            ),
            SizeError::Mask { actual, required } => write!(
                f,
                "Mask length is invalid for channel count, got: {}, required: {}",
                actual, required
            ),
        }
    }
}

macro_rules! implement_size_getters {
    () => {
        fn channels(&self) -> usize {
            self.channels
        }

        fn frames(&self) -> usize {
            self.frames
        }
    };
}
pub(crate) use implement_size_getters;

macro_rules! check_slice_length {
    ($channels:expr , $frames:expr, $length:expr ) => {
        // Saturating, so an overflowing product becomes `usize::MAX` and the
        // check fails instead of wrapping to a small value and passing.
        let required = $frames.saturating_mul($channels);
        if $length < required {
            return Err(SizeError::Total {
                actual: $length,
                required,
            });
        }
    };
    ($channels:expr , $frames:expr, $length:expr, $elements_per_sample:expr) => {
        let required = $frames
            .saturating_mul($channels)
            .saturating_mul($elements_per_sample);
        if $length < required {
            return Err(SizeError::Total {
                actual: $length,
                required,
            });
        }
    };
}
pub(crate) use check_slice_length;

#[cfg(test)]
mod tests {
    use audioadapter::AdapterMut;

    fn prepare_test_data(buffer: &mut dyn AdapterMut<u32>) {
        for channel in 0..buffer.channels() {
            for frame in 0..buffer.frames() {
                let value = (100 * channel + frame) as u32;
                buffer.write_sample(channel, frame, &value);
            }
        }
    }

    pub(crate) fn check_copy_within(buffer: &mut dyn AdapterMut<u32>) {
        assert!(buffer.channels() > 1, "Too few channels to run tests");
        assert!(buffer.frames() > 8, "Too few frames to run test");
        // copy forward, no overlap
        prepare_test_data(buffer);
        assert_eq!(buffer.copy_frames_within(1, 5, 3), Some(3));
        check_copy_result(buffer, 1, 5, 3);

        // copy backwards, no overlap
        prepare_test_data(buffer);
        assert_eq!(buffer.copy_frames_within(5, 1, 3), Some(3));
        check_copy_result(buffer, 5, 1, 3);

        // copy forward, with overlap
        prepare_test_data(buffer);
        assert_eq!(buffer.copy_frames_within(1, 3, 5), Some(5));
        check_copy_result(buffer, 1, 3, 5);

        // copy backwards, with overlap
        prepare_test_data(buffer);
        assert_eq!(buffer.copy_frames_within(3, 1, 5), Some(5));
        check_copy_result(buffer, 3, 1, 5);
    }

    fn check_copy_result(buffer: &dyn AdapterMut<u32>, src: usize, dest: usize, count: usize) {
        for channel in 0..buffer.channels() {
            for frame in 0..buffer.frames() {
                let copied_frame = if frame >= dest && frame < dest + count {
                    frame + src - dest
                } else {
                    frame
                };
                let expected_value = (100 * channel + copied_frame) as u32;
                assert_eq!(
                    buffer.read_sample(channel, frame),
                    Some(expected_value),
                    "Wrong value at ch {}, frame {}",
                    channel,
                    frame
                );
            }
        }
    }
}
