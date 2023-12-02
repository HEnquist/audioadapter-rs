use num_traits::{Num, ToPrimitive};

use crate::Adapter;

/// A trait providing methods to calculate the RMS and peak-to-peak values of a channel or frame.
/// This requires that the samples are of a numerical type, that implement the
/// [num_traits::ToPrimitive], [num_traits::Num] and [core::cmp::PartialOrd] traits.
/// This includes all the built in numerical types such as `i16`, `i32`, `f32` etc.
pub trait AdapterStats<'a, T>: Adapter<'a, T>
where
    T: Clone + ToPrimitive + Num + PartialOrd + 'a,
{
    /// Calculate the RMS value of the given channel.
    /// The result is returned as `f64`.
    fn channel_rms(&self, channel: usize) -> f64 {
        let mut square_sum = 0.0;
        if self.frames() == 0 || self.channels() == 0 {
            return 0.0;
        }
        for frame in 0..self.frames() {
            square_sum += self
                .read_sample(channel, frame)
                .unwrap_or(T::zero())
                .to_f64()
                .unwrap_or_default()
                .powi(2);
        }
        (square_sum / self.frames() as f64).sqrt()
    }

    /// Calculate the RMS value of the given channel.
    /// The result is returned as `f64`.
    fn frame_rms(&self, frame: usize) -> f64 {
        let mut square_sum = 0.0;
        if self.frames() == 0 || self.channels() == 0 {
            return 0.0;
        }
        for channel in 0..self.channels() {
            square_sum += self
                .read_sample(channel, frame)
                .unwrap_or(T::zero())
                .to_f64()
                .unwrap_or_default()
                .powi(2);
        }
        (square_sum / self.frames() as f64).sqrt()
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

impl<'a, T, U> AdapterStats<'a, T> for U
where
    T: Clone + ToPrimitive + Num + PartialOrd + 'a,
    U: Adapter<'a, T>,
{
}

//   _____         _
//  |_   _|__  ___| |_ ___
//    | |/ _ \/ __| __/ __|
//    | |  __/\__ \ |_\__ \
//    |_|\___||___/\__|___/

#[cfg(test)]
mod tests {
    use crate::direct::SequentialSlice;
    use crate::stats::AdapterStats;

    #[test]
    fn stats_integer() {
        let data = [1_i32, -1, 1, -1, 1, -1, 1, -1];
        let buffer = SequentialSlice::new(&data, 2, 4).unwrap();
        assert_eq!(buffer.channel_rms(0), 1.0);
        assert_eq!(buffer.channel_min_and_max(0), (-1, 1));
        assert_eq!(buffer.channel_peak_to_peak(0), 2.0);
    }

    #[test]
    fn stats_float() {
        let data = [1.0_f32, -1.0, 1.0, -1.0, 1.0, -1.0, 1.0, -1.0];
        let buffer = SequentialSlice::new(&data, 2, 4).unwrap();
        assert_eq!(buffer.channel_rms(0), 1.0);
        assert_eq!(buffer.channel_min_and_max(0), (-1.0, 1.0));
        assert_eq!(buffer.channel_peak_to_peak(0), 2.0);
    }
}
