#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "std"), no_std)]

/// Wrappers providing direct access to samples in buffers.
pub mod direct;
/// Wrappers providing float conversion of numeric values
/// stored both directly and as raw bytes.
pub mod number_to_float;
/// Wrappers that store their data in an owned vector.
#[cfg(feature = "std")]
pub mod owned;
/// The traits for accessing samples in buffers.
mod traits;

/// Type conversion of samples values.
pub mod sample;

/// Extensions to the standard [std::io::Read] and [std::io::Write] traits.
pub mod readwrite;

/// Calculate statistics for adapters with numerical sample types
pub mod stats;

/// Read-only iterators
mod iterators;

mod slicetools;

#[cfg(feature = "std")]
use std::error::Error;
#[cfg(feature = "std")]
use std::fmt;

pub use traits::{Adapter, AdapterMut};

pub use iterators::AdapterIterators;

#[cfg(feature = "audio")]
pub mod audio;

pub mod adapter_to_float;

/// Error returned when the wrapped data structure has the wrong dimensions,
/// typically that it is too short.
#[derive(Debug)]
pub enum SizeError {
    Channel {
        index: usize,
        actual: usize,
        required: usize,
    },
    Frame {
        index: usize,
        actual: usize,
        required: usize,
    },
    Total {
        actual: usize,
        required: usize,
    },
    Mask {
        actual: usize,
        required: usize,
    },
}

#[cfg(feature = "std")]
impl Error for SizeError {}

#[cfg(feature = "std")]
impl fmt::Display for SizeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let desc = match self {
            SizeError::Channel {
                index,
                actual,
                required,
            } => format!(
                "Buffer for channel {} is too short, got: {}, required: {}",
                index, actual, required
            ),
            SizeError::Frame {
                index,
                actual,
                required,
            } => format!(
                "Buffer for frame {} is too short, got: {}, required: {}",
                index, actual, required
            ),
            SizeError::Total { actual, required } => format!(
                "Buffer is too short, got: {}, required: {}",
                actual, required
            ),
            SizeError::Mask { actual, required } => format!(
                "Mask is wrong length, got: {}, required: {}",
                actual, required
            ),
        };
        write!(f, "{}", &desc)
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
        if $length < $frames * $channels {
            return Err(SizeError::Total {
                actual: $length,
                required: $frames * $channels,
            });
        }
    };
    ($channels:expr , $frames:expr, $length:expr, $elements_per_sample:expr) => {
        if $length < $frames * $channels * $elements_per_sample {
            return Err(SizeError::Total {
                actual: $length,
                required: $frames * $channels * $elements_per_sample,
            });
        }
    };
}
pub(crate) use check_slice_length;

#[cfg(test)]
mod tests {
    use crate::AdapterMut;

    fn prepare_test_data(buffer: &mut dyn AdapterMut<u32>) {
        for channel in 0..buffer.channels() {
            for frame in 0..buffer.frames() {
                let value = (100 * channel + frame) as u32;
                buffer.write_sample(channel, frame, &value);
            }
        }
    }

    pub(crate) fn check_copy_within(buffer: &mut dyn AdapterMut<u32>) {
        assert!(buffer.channels() > 1, "Too few chanels to run tests");
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
