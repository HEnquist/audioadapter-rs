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

/// Calculate statistics for adapters with numerical sample types
pub mod stats;

/// Read-only iterators
mod iterators;

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
