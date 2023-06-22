#![doc = include_str!("../README.md")]

/// Wrappers providing conversion between raw bytes and floating point samples.
pub mod converting;
/// Wrappers providing direct access to samples in buffers.
pub mod direct;
/// The traits for accessing samples in buffers.
pub mod traits;

use std::error::Error;
use std::fmt;

mod iterators;
mod stats;
pub use iterators::{
    ChannelSamples, ChannelSamplesMut, Channels, ChannelsMut, FrameSamples, FrameSamplesMut,
    Frames, FramesMut,
};
pub use stats::Numeric;

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

impl Error for SizeError {}

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

#[macro_export]
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
#[macro_export]
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

#[macro_export]
macro_rules! check_slice_and_vec_length {
    ($buf:expr, $channels:expr, $frames:expr, sequential) => {
        if $buf.len() < $channels {
            return Err(SizeError::Frame {
                index: 0,
                actual: $buf.len(),
                required: $channels,
            });
        }
        for (idx, chan) in $buf.iter().enumerate() {
            if chan.len() < $frames {
                return Err(SizeError::Channel {
                    index: idx,
                    actual: chan.len(),
                    required: $frames,
                });
            }
        }
    };
    ($buf:expr, $channels:expr, $frames:expr, interleaved) => {
        if $buf.len() < $frames {
            return Err(SizeError::Channel {
                index: 0,
                actual: $buf.len(),
                required: $frames,
            });
        }
        for (idx, frame) in $buf.iter().enumerate() {
            if frame.len() < $channels {
                return Err(SizeError::Frame {
                    index: idx,
                    actual: frame.len(),
                    required: $channels,
                });
            }
        }
    };
}
