use num_traits::{Num, ToPrimitive};

use crate::Adapter;

/// A simple implementation of Newton's method for calculating the square root of a number.
/// This is used to avoid depending on `std`, until math support in core is stable.
/// See: <https://doc.rust-lang.org/core/f64/math/fn.sqrt.html>
///
/// This is not a fully standards-equivalent replacement for `f64::sqrt`.
/// It is intentionally simplified for this crate's RMS/statistics use case,
/// where inputs are expected to be finite and non-negative in normal operation.
///
/// Behavior:
/// - `NaN` is propagated.
/// - `+inf` returns `+inf`.
/// - `-inf` returns `0.0`.
/// - Negative finite values return `0.0`.
fn sqrt_newton(value: f64) -> f64 {
    if value.is_nan() {
        return value;
    }
    if value.is_infinite() {
        return if value.is_sign_positive() {
            f64::INFINITY
        } else {
            0.0
        };
    }
    if value <= 0.0 {
        return 0.0;
    }

    // Get an initial guess using the exponent of the floating point representation.
    let mut estimate = f64::from_bits((value.to_bits() + (1023_u64 << 52)) >> 1);

    // Perform 5 iterations of Newton's method to refine the estimate.
    for _ in 0..5 {
        estimate = 0.5 * (estimate + value / estimate);
    }

    estimate
}

/// A trait providing methods to calculate the RMS and peak-to-peak values of a channel or frame.
/// This requires that the samples are of a numerical type, that implement the
/// [num_traits::ToPrimitive], [num_traits::Num] and [core::cmp::PartialOrd] traits.
/// This includes all the built in numerical types such as `i16`, `i32`, `f32` etc.
pub trait AdapterStats<T>: Adapter<T>
where
    T: Clone + ToPrimitive + Num + PartialOrd,
{
    /// Calculate the RMS value of the given channel.
    /// The result is returned as `f64`.
    fn channel_rms(&self, channel: usize) -> f64 {
        let mut square_sum = 0.0;
        if self.frames() == 0 || self.channels() == 0 {
            return 0.0;
        }
        for frame in 0..self.frames() {
            let sample = self
                .read_sample(channel, frame)
                .unwrap_or(T::zero())
                .to_f64()
                .unwrap_or_default();
            square_sum += sample * sample;
        }
        sqrt_newton(square_sum / self.frames() as f64)
    }

    /// Calculate the RMS value of the given channel.
    /// The result is returned as `f64`.
    fn frame_rms(&self, frame: usize) -> f64 {
        let mut square_sum = 0.0;
        if self.frames() == 0 || self.channels() == 0 {
            return 0.0;
        }
        for channel in 0..self.channels() {
            let sample = self
                .read_sample(channel, frame)
                .unwrap_or(T::zero())
                .to_f64()
                .unwrap_or_default();
            square_sum += sample * sample;
        }
        sqrt_newton(square_sum / self.channels() as f64)
    }

    /// Calculate the peak-to-peak value of the given channel.
    /// The result is returned as a tuple `(min, max)`
    /// with values of the same type as the samples.
    fn channel_min_and_max(&self, channel: usize) -> (T, T) {
        let mut min = T::zero();
        let mut max = T::zero();
        if self.frames() == 0 || self.channels() == 0 {
            return (T::zero(), T::zero());
        }
        for frame in 0..self.frames() {
            let sample = self.read_sample(channel, frame).unwrap_or(T::zero());
            if sample < min {
                min = sample;
            } else if sample > max {
                max = sample;
            }
        }
        (min, max)
    }

    /// Calculate the peak-to-peak value of the given channel.
    /// The result is returned as `f64`.
    fn channel_peak_to_peak(&self, channel: usize) -> f64 {
        let (min, max) = self.channel_min_and_max(channel);
        max.to_f64().unwrap_or_default() - min.to_f64().unwrap_or_default()
    }

    /// Calculate the peak-to-peak value of the given frame.
    /// The result is returned as a tuple `(min, max)`
    /// with values of the same type as the samples.
    fn frame_min_and_max(&self, frame: usize) -> (T, T) {
        let mut min = T::zero();
        let mut max = T::zero();
        if self.frames() == 0 || self.channels() == 0 {
            return (T::zero(), T::zero());
        }
        for channel in 0..self.channels() {
            let sample = self.read_sample(channel, frame).unwrap_or(T::zero());
            if sample < min {
                min = sample;
            } else if sample > max {
                max = sample;
            }
        }
        (min, max)
    }

    /// Calculate the peak-to-peak value of the given frame.
    /// The result is returned as `f64`.
    fn frame_peak_to_peak(&self, frame: usize) -> f64 {
        let (min, max) = self.frame_min_and_max(frame);
        max.to_f64().unwrap_or_default() - min.to_f64().unwrap_or_default()
    }
}

impl<T, U> AdapterStats<T> for U
where
    T: Clone + ToPrimitive + Num + PartialOrd,
    U: Adapter<T>,
{
}

//   _____         _
//  |_   _|__  ___| |_ ___
//    | |/ _ \/ __| __/ __|
//    | |  __/\__ \ |_\__ \
//    |_|\___||___/\__|___/

#[cfg(test)]
mod tests {
    extern crate alloc;

    use crate::stats::AdapterStats;
    use crate::tests::MinimalAdapter;
    use alloc::vec;

    #[test]
    fn stats_integer() {
        let data = vec![1_i32, 1, -1, -1, 1, 1, -1, -1];
        let buffer = MinimalAdapter::new_from_vec(data, 2, 4);
        assert_eq!(buffer.channel_rms(0), 1.0);
        assert_eq!(buffer.channel_min_and_max(0), (-1, 1));
        assert_eq!(buffer.channel_peak_to_peak(0), 2.0);
    }

    #[test]
    fn stats_float() {
        let data = vec![1.0_f32, 1.0, -1.0, -1.0, 1.0, 1.0, -1.0, -1.0];
        let buffer = MinimalAdapter::new_from_vec(data, 2, 4);
        assert_eq!(buffer.channel_rms(0), 1.0);
        assert_eq!(buffer.channel_min_and_max(0), (-1.0, 1.0));
        assert_eq!(buffer.channel_peak_to_peak(0), 2.0);
    }

    #[test]
    fn stats_frame_integer() {
        let data = vec![-1_i32, 1, -1, 1, -1, 1, -1, 1];
        let buffer = MinimalAdapter::new_from_vec(data, 2, 4);
        assert_eq!(buffer.frame_rms(0), 1.0);
        assert_eq!(buffer.frame_min_and_max(0), (-1, 1));
        assert_eq!(buffer.frame_peak_to_peak(0), 2.0);
    }

    #[test]
    fn stats_frame_float() {
        let data = vec![-1.0_f32, 1.0, -1.0, 1.0, -1.0, 1.0, -1.0, 1.0];
        let buffer = MinimalAdapter::new_from_vec(data, 2, 4);
        assert_eq!(buffer.frame_rms(0), 1.0);
        assert_eq!(buffer.frame_min_and_max(0), (-1.0, 1.0));
        assert_eq!(buffer.frame_peak_to_peak(0), 2.0);
    }

    #[test]
    fn sqrt_newton_accuracy() {
        let test_values: [f64; 12] = [
            1.0e-12_f64,
            1.0e-9_f64,
            1.0e-6_f64,
            1.0e-3_f64,
            0.1_f64,
            0.5_f64,
            1.0_f64,
            2.0_f64,
            10.0_f64,
            100.0_f64,
            1.0e6_f64,
            1.0e12_f64,
        ];

        for value in test_values {
            let expected = value.sqrt();
            let actual = super::sqrt_newton(value);
            let rel_err = (actual - expected).abs() / expected.max(1.0);
            assert!(
                rel_err < 1.0e-12,
                "value={value}, expected={expected}, actual={actual}, rel_err={rel_err}"
            );
        }
    }

    #[test]
    fn sqrt_newton_special_values() {
        assert!(super::sqrt_newton(f64::NAN).is_nan());
        assert_eq!(super::sqrt_newton(f64::INFINITY), f64::INFINITY);
        assert_eq!(super::sqrt_newton(f64::NEG_INFINITY), 0.0);
    }

    #[test]
    fn sqrt_newton_subnormal_value() {
        let values = [
            f64::from_bits(1),
            f64::from_bits(f64::MIN_POSITIVE.to_bits() - 1),
        ];
        for value in values {
            let expected = value.sqrt();
            let actual = super::sqrt_newton(value);
            let rel_err = (actual - expected).abs() / expected.max(1.0);
            assert!(
                rel_err < 1.0e-12,
                "value={value:e}, expected={expected:e}, actual={actual:e}, rel_err={rel_err:e}"
            );
        }
    }
}
