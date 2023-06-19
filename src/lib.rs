#![doc = include_str!("../README.md")]

/// Wrappers providing conversion between raw bytes and floating point samples.
pub mod converting;
/// Wrappers providing direct access to samples in buffers.
pub mod direct;
/// The traits for accessing samples in buffers.
pub mod traits;

use std::error;
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
pub struct BufferSizeError {
    pub desc: String,
}

impl fmt::Display for BufferSizeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.desc)
    }
}

impl error::Error for BufferSizeError {
    fn description(&self) -> &str {
        &self.desc
    }
}

impl BufferSizeError {
    pub fn new(desc: &str) -> Self {
        BufferSizeError {
            desc: desc.to_owned(),
        }
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
    ($channels:expr , $frames:expr, $length:expr) => {
        if $length < $frames * $channels {
            return Err(BufferSizeError {
                desc: format!("Slice is too short, {} < {}", $length, $frames * $channels),
            });
        }
    };
    ($channels:expr , $frames:expr, $length:expr, $elements_per_sample:expr) => {
        if $length < $frames * $channels * $elements_per_sample {
            return Err(BufferSizeError {
                desc: format!(
                    "Slice is too short, {} < {}",
                    $length,
                    $frames * $channels * $elements_per_sample
                ),
            });
        }
    };
}

#[macro_export]
macro_rules! check_slice_and_vec_length {
    ($buf:expr, $channels:expr, $frames:expr, sequential) => {
        if $buf.len() < $channels {
            return Err(BufferSizeError {
                desc: format!("Too few channels, {} < {}", $buf.len(), $channels),
            });
        }
        for (idx, chan) in $buf.iter().enumerate() {
            if chan.len() < $frames {
                return Err(BufferSizeError {
                    desc: format!("Channel {} is too short, {} < {}", idx, chan.len(), $frames),
                });
            }
        }
    };
    ($buf:expr, $channels:expr, $frames:expr, interleaved) => {
        if $buf.len() < $frames {
            return Err(BufferSizeError {
                desc: format!("Too few frames, {} < {}", $buf.len(), $frames),
            });
        }
        for (idx, frame) in $buf.iter().enumerate() {
            if frame.len() < $channels {
                return Err(BufferSizeError {
                    desc: format!(
                        "Frame {} is too short, {} < {}",
                        idx,
                        frame.len(),
                        $channels
                    ),
                });
            }
        }
    };
}
