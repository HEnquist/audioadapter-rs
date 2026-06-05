#![doc = include_str!("../README.md")]
#![no_std]

/// The traits for accessing samples in buffers.
mod traits;
pub use traits::{Adapter, AdapterMut};

/// Calculate statistics for adapters with numerical sample types
pub mod stats;

/// Read-only iterators
mod iterators;

pub use iterators::AdapterIterators;

#[cfg(feature = "audio")]
pub mod audio;

#[cfg(any(test, feature = "test-utils"))]
pub mod tests {
    extern crate alloc;

    use crate::{Adapter, AdapterMut};
    use alloc::vec;
    use alloc::vec::Vec;
    use num_traits::{NumCast, float::FloatCore};

    /// Minimal implementation of an Adapter based on a vec
    /// intended for testing purposes.
    pub struct MinimalAdapter<U> {
        buf: Vec<U>,
        frames: usize,
        channels: usize,
    }

    impl<T> MinimalAdapter<T>
    where
        T: Clone,
    {
        pub fn new_from_vec(buf: Vec<T>, channels: usize, frames: usize) -> Self {
            Self {
                buf,
                frames,
                channels,
            }
        }
    }

    unsafe impl<T> Adapter<T> for MinimalAdapter<T>
    where
        T: Clone,
    {
        unsafe fn read_sample_unchecked(&self, channel: usize, frame: usize) -> T {
            let index = frame * self.channels + channel;
            unsafe { self.buf.get_unchecked(index).clone() }
        }

        fn channels(&self) -> usize {
            self.channels
        }

        fn frames(&self) -> usize {
            self.frames
        }
    }

    unsafe impl<T> AdapterMut<T> for MinimalAdapter<T>
    where
        T: Clone,
    {
        unsafe fn write_sample_unchecked(
            &mut self,
            channel: usize,
            frame: usize,
            value: &T,
        ) -> bool {
            let index = frame * self.channels + channel;
            unsafe {
                *self.buf.get_unchecked_mut(index) = value.clone();
            }
            false
        }
    }

    /// A generic test function to verify the implementation of `Adapter` and `AdapterMut` traits.
    ///
    /// It takes a mutable reference to an adapter and runs a series of tests.
    /// The adapter is expected to have at least 2 channels and 4 frames.
    /// The sample type `T` must support `Default`, `Clone`, `PartialEq`, `Debug`, and be convertible to and from `usize`.
    pub fn test_adapter_mut_methods<T>(buffer: &mut dyn AdapterMut<T>)
    where
        T: Default + Clone + PartialEq + core::fmt::Debug + From<usize> + Into<usize>,
    {
        // Ensure buffer is large enough for tests
        assert!(
            buffer.channels() >= 2,
            "Buffer must have at least 2 channels for this test"
        );
        assert!(
            buffer.frames() >= 4,
            "Buffer must have at least 4 frames for this test"
        );

        // --- Test `fill_with` and `read_sample` ---
        buffer.fill_with(&T::from(42));
        assert_eq!(buffer.read_sample(0, 0), Some(T::from(42)));
        assert_eq!(buffer.read_sample(1, 1), Some(T::from(42)));
        // Test OOB read
        assert_eq!(buffer.read_sample(buffer.channels(), 0), None);
        assert_eq!(buffer.read_sample(0, buffer.frames()), None);

        // --- Test `write_sample` ---
        assert_eq!(buffer.write_sample(0, 0, &T::from(100)), Some(false));
        assert_eq!(buffer.read_sample(0, 0), Some(T::from(100)));
        // Test OOB write
        assert_eq!(
            buffer.write_sample(buffer.channels(), 0, &T::from(101)),
            None
        );
        assert_eq!(buffer.write_sample(0, buffer.frames(), &T::from(102)), None);

        // --- Test `fill_channel_with` ---
        buffer.fill_channel_with(1, &T::from(99)).unwrap();
        assert_eq!(buffer.read_sample(1, 0), Some(T::from(99)));
        assert_eq!(buffer.read_sample(1, 1), Some(T::from(99)));
        assert_eq!(buffer.read_sample(0, 0), Some(T::from(100))); // Other channel unaffected

        // --- Test `fill_frame_with` ---
        buffer.fill_frame_with(2, &T::from(88)).unwrap();
        assert_eq!(buffer.read_sample(0, 2), Some(T::from(88)));
        assert_eq!(buffer.read_sample(1, 2), Some(T::from(88)));
        assert_eq!(buffer.read_sample(1, 1), Some(T::from(99))); // Other frame unaffected

        // --- Test `fill_frames_with` ---
        buffer.fill_frames_with(0, 2, &T::from(77)).unwrap();
        assert_eq!(buffer.read_sample(0, 0), Some(T::from(77)));
        assert_eq!(buffer.read_sample(1, 1), Some(T::from(77)));
        assert_eq!(buffer.read_sample(0, 2), Some(T::from(88))); // Unaffected frame

        // Reset for next tests
        for c in 0..buffer.channels() {
            for f in 0..buffer.frames() {
                buffer.write_sample(c, f, &T::from(c * 10 + f));
            }
        }
        // Expected: ch0: [0, 1, 2, 3, ...], ch1: [10, 11, 12, 13, ...]

        // --- Test `copy_from_channel_to_slice` ---
        let mut slice_ch = vec![T::default(); 2];
        let copied = buffer.copy_from_channel_to_slice(1, 1, &mut slice_ch);
        assert_eq!(copied, 2);
        assert_eq!(slice_ch, vec![T::from(11), T::from(12)]);

        // --- Test `copy_from_frame_to_slice` ---
        let mut slice_fr = vec![T::default(); 2];
        let copied = buffer.copy_from_frame_to_slice(2, 0, &mut slice_fr);
        assert_eq!(copied, 2);
        assert_eq!(slice_fr, vec![T::from(2), T::from(12)]);

        // --- Test `copy_from_slice_to_channel` ---
        let slice_to_ch = vec![T::from(101), T::from(102)];
        let (copied, clipped) = buffer.copy_from_slice_to_channel(0, 2, &slice_to_ch);
        assert_eq!(copied, 2);
        assert_eq!(clipped, 0);
        assert_eq!(buffer.read_sample(0, 2), Some(T::from(101)));
        assert_eq!(buffer.read_sample(0, 3), Some(T::from(102)));

        // --- Test `copy_from_slice_to_frame` ---
        let slice_to_fr = vec![T::from(201)];
        let (copied, clipped) = buffer.copy_from_slice_to_frame(3, 1, &slice_to_fr);
        assert_eq!(copied, 1);
        assert_eq!(clipped, 0);
        assert_eq!(buffer.read_sample(1, 3), Some(T::from(201)));

        // --- Test `copy_sample_within` ---
        // Before: (0,0) is 0, (1,1) is 11
        assert!(buffer.copy_sample_within(0, 0, 1, 1));
        // After: (0,0) is 0, (1,1) is 0
        assert_eq!(buffer.read_sample(1, 1), Some(T::from(0)));

        // --- Test `swap_samples` ---
        // Before: (0,1) is 1, (1,0) is 10
        assert!(buffer.swap_samples(0, 1, 1, 0));
        // After: (0,1) is 10, (1,0) is 1
        assert_eq!(buffer.read_sample(0, 1), Some(T::from(10)));
        assert_eq!(buffer.read_sample(1, 0), Some(T::from(1)));

        // --- Test `copy_frames_within` ---
        // Before: F0:[0, 1], F1:[10, 0], F2:[101, 12]
        buffer.copy_frames_within(0, 1, 2).unwrap();
        // After: F0:[0, 1], F1:[0, 1], F2:[10, 0]
        assert_eq!(buffer.read_sample(0, 1), Some(T::from(0)));
        assert_eq!(buffer.read_sample(1, 1), Some(T::from(1)));
        assert_eq!(buffer.read_sample(0, 2), Some(T::from(10)));
        assert_eq!(buffer.read_sample(1, 2), Some(T::from(0)));
    }

    /// A generic test function to verify the implementation of `Adapter` and `AdapterMut` traits for float types.
    ///
    /// It takes a mutable reference to an adapter and runs a series of tests.
    /// The adapter is expected to have at least 2 channels and 4 frames.
    /// The sample type `T` must be a floating-point type implementing `FloatCore`, `NumCast`, `Default`, `Clone`, `PartialEq`, and `Debug`.
    pub fn test_float_adapter_mut_methods<T>(buffer: &mut dyn AdapterMut<T>)
    where
        T: FloatCore + NumCast + Default + Clone + PartialEq + core::fmt::Debug,
    {
        // Helper for approximate float comparison
        let assert_approx_eq = |a: T, b: T, message: &str| {
            let epsilon = NumCast::from(1e-6f64).unwrap();
            assert!(
                (a - b).abs() < epsilon,
                "{} (left: {:?}, right: {:?})",
                message,
                a,
                b
            );
        };

        let assert_slice_approx_eq = |a: &[T], b: &[T], message: &str| {
            assert_eq!(a.len(), b.len(), "{} (slice lengths differ)", message);
            for (i, (val_a, val_b)) in a.iter().zip(b.iter()).enumerate() {
                let msg = alloc::format!("{} (element {})", message, i);
                assert_approx_eq(*val_a, *val_b, &msg);
            }
        };

        // Ensure buffer is large enough for tests
        assert!(
            buffer.channels() >= 2,
            "Buffer must have at least 2 channels for this test"
        );
        assert!(
            buffer.frames() >= 4,
            "Buffer must have at least 4 frames for this test"
        );

        // --- Test `fill_with` and `read_sample` ---
        buffer.fill_with(&NumCast::from(0.42f64).unwrap());
        assert_approx_eq(
            buffer.read_sample(0, 0).unwrap(),
            NumCast::from(0.42f64).unwrap(),
            "fill_with value mismatch",
        );
        assert_approx_eq(
            buffer.read_sample(1, 1).unwrap(),
            NumCast::from(0.42f64).unwrap(),
            "fill_with value mismatch",
        );
        // Test OOB read
        assert_eq!(buffer.read_sample(buffer.channels(), 0), None);
        assert_eq!(buffer.read_sample(0, buffer.frames()), None);

        // --- Test `write_sample` ---
        assert_eq!(
            buffer.write_sample(0, 0, &NumCast::from(0.1f64).unwrap()),
            Some(false)
        );
        assert_approx_eq(
            buffer.read_sample(0, 0).unwrap(),
            NumCast::from(0.1f64).unwrap(),
            "write_sample value mismatch",
        );
        // Test OOB write
        assert_eq!(
            buffer.write_sample(buffer.channels(), 0, &NumCast::from(0.101f64).unwrap()),
            None
        );
        assert_eq!(
            buffer.write_sample(0, buffer.frames(), &NumCast::from(0.102f64).unwrap()),
            None
        );

        // --- Test `fill_channel_with` ---
        buffer
            .fill_channel_with(1, &NumCast::from(0.99f64).unwrap())
            .unwrap();
        assert_approx_eq(
            buffer.read_sample(1, 0).unwrap(),
            NumCast::from(0.99f64).unwrap(),
            "fill_channel_with value mismatch",
        );
        assert_approx_eq(
            buffer.read_sample(1, 1).unwrap(),
            NumCast::from(0.99f64).unwrap(),
            "fill_channel_with value mismatch",
        );
        assert_approx_eq(
            buffer.read_sample(0, 0).unwrap(),
            NumCast::from(0.1f64).unwrap(),
            "Other channel should be unaffected",
        ); // Other channel unaffected

        // --- Test `fill_frame_with` ---
        buffer
            .fill_frame_with(2, &NumCast::from(0.88f64).unwrap())
            .unwrap();
        assert_approx_eq(
            buffer.read_sample(0, 2).unwrap(),
            NumCast::from(0.88f64).unwrap(),
            "fill_frame_with value mismatch",
        );
        assert_approx_eq(
            buffer.read_sample(1, 2).unwrap(),
            NumCast::from(0.88f64).unwrap(),
            "fill_frame_with value mismatch",
        );
        assert_approx_eq(
            buffer.read_sample(1, 1).unwrap(),
            NumCast::from(0.99f64).unwrap(),
            "Other frame should be unaffected",
        ); // Other frame unaffected

        // --- Test `fill_frames_with` ---
        buffer
            .fill_frames_with(0, 2, &NumCast::from(0.77f64).unwrap())
            .unwrap();
        assert_approx_eq(
            buffer.read_sample(0, 0).unwrap(),
            NumCast::from(0.77f64).unwrap(),
            "fill_frames_with value mismatch",
        );
        assert_approx_eq(
            buffer.read_sample(1, 1).unwrap(),
            NumCast::from(0.77f64).unwrap(),
            "fill_frames_with value mismatch",
        );
        assert_approx_eq(
            buffer.read_sample(0, 2).unwrap(),
            NumCast::from(0.88f64).unwrap(),
            "Unaffected frame should be unaffected",
        ); // Unaffected frame

        // Reset for next tests
        for c in 0..buffer.channels() {
            for f in 0..buffer.frames() {
                buffer.write_sample(c, f, &NumCast::from((c * 10 + f) as f64 / 100.0).unwrap());
            }
        }
        // Expected: ch0: [0.00, 0.01, 0.02, 0.03, ...], ch1: [0.10, 0.11, 0.12, 0.13, ...]

        // --- Test `copy_from_channel_to_slice` ---
        let mut slice_ch = vec![T::default(); 2];
        let copied = buffer.copy_from_channel_to_slice(1, 1, &mut slice_ch);
        assert_eq!(copied, 2);
        assert_slice_approx_eq(
            &slice_ch,
            &[
                NumCast::from(0.11f64).unwrap(),
                NumCast::from(0.12f64).unwrap(),
            ],
            "copy_from_channel_to_slice mismatch",
        );

        // --- Test `copy_from_frame_to_slice` ---
        let mut slice_fr = vec![T::default(); 2];
        let copied = buffer.copy_from_frame_to_slice(2, 0, &mut slice_fr);
        assert_eq!(copied, 2);
        assert_slice_approx_eq(
            &slice_fr,
            &[
                NumCast::from(0.02f64).unwrap(),
                NumCast::from(0.12f64).unwrap(),
            ],
            "copy_from_frame_to_slice mismatch",
        );

        // --- Test `copy_sample_within` ---
        // Before: (0,0) is 0.0, (1,1) is 0.11
        assert!(buffer.copy_sample_within(0, 0, 1, 1));
        // After: (0,0) is 0.0, (1,1) is 0.0
        assert_approx_eq(
            buffer.read_sample(1, 1).unwrap(),
            NumCast::from(0.0f64).unwrap(),
            "copy_sample_within value mismatch",
        );

        // --- Test `swap_samples` ---
        // Before: (0,1) is 0.01, (1,0) is 0.10
        assert!(buffer.swap_samples(0, 1, 1, 0));
        // After: (0,1) is 0.10, (1,0) is 0.01
        assert_approx_eq(
            buffer.read_sample(0, 1).unwrap(),
            NumCast::from(0.10f64).unwrap(),
            "swap_samples value mismatch on sample A",
        );
        assert_approx_eq(
            buffer.read_sample(1, 0).unwrap(),
            NumCast::from(0.01f64).unwrap(),
            "swap_samples value mismatch on sample B",
        );
    }

    #[test]
    fn test_vec_adapter() {
        let mut buffer = MinimalAdapter::new_from_vec(vec![0; 8], 2, 4);
        test_adapter_mut_methods(&mut buffer);
    }
}
