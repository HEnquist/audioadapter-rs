use num_traits::{Num, ToPrimitive};

use crate::traits::AudioBuffer;

/// A trait providing methods to calculate the RMS and peak-to-peak values of a channel or frame.
/// This requires that the samples are of a numerical type, that also implement the
/// [num_traits::ToPrimitive], [num_traits::Num] and [core::cmp::PartialOrd] traits.
/// This includes all the built in numerical types such as `i16`, `i32`, `f32` etc.
pub trait AudioBufferStats<'a, T>: AudioBuffer<'a, T>
where
    T: Clone + ToPrimitive + Num + PartialOrd + 'a,
{
    /// Calculate the RMS value of the given channel.
    /// The result is returned as `f64`.
    fn channel_rms(&self, channel: usize) -> Option<f64> {
        let (square_sum, nbr_values) = self.iter_channel(channel)?.fold((0.0, 0), |acc, x| {
            (acc.0 + x.to_f64().unwrap_or_default().powi(2), acc.1 + 1)
        });
        if nbr_values == 0 {
            return None;
        }
        Some((square_sum / nbr_values as f64).sqrt())
    }

    /// Calculate the RMS value of the given channel.
    /// The result is returned as `f64`.
    fn frame_rms(&self, frame: usize) -> Option<f64> {
        let (square_sum, nbr_values) = self.iter_frame(frame)?.fold((0.0, 0), |acc, x| {
            (acc.0 + x.to_f64().unwrap_or_default().powi(2), acc.1 + 1)
        });
        if nbr_values == 0 {
            return None;
        }
        Some((square_sum / nbr_values as f64).sqrt())
    }

    /// Calculate the peak-to-peak value of the given channel.
    /// The result is returned as the same type as the samples.
    fn channel_peak_to_peak(&self, channel: usize) -> Option<T> {
        let [min, max] = self
            .iter_channel(channel)?
            .fold([T::zero(), T::zero()], |mut acc, x| {
                if *x < acc[0] {
                    acc[0] = x.clone();
                } else if *x > acc[1] {
                    acc[1] = x.clone();
                }
                acc
            });
        Some(max - min)
    }

    /// Calculate the peak-to-peak value of the given frame.
    /// The result is returned as the same type as the samples.
    fn frame_peak_to_peak(&self, frame: usize) -> Option<T> {
        let [min, max] = self
            .iter_frame(frame)?
            .fold([T::zero(), T::zero()], |mut acc, x| {
                if *x < acc[0] {
                    acc[0] = x.clone();
                } else if *x > acc[1] {
                    acc[1] = x.clone();
                }
                acc
            });
        Some(max - min)
    }
}

impl<'a, T, U> AudioBufferStats<'a, T> for U
where
    T: Clone + ToPrimitive + Num + PartialOrd + 'a,
    U: AudioBuffer<'a, T>,
{
}

//   _____         _
//  |_   _|__  ___| |_ ___
//    | |/ _ \/ __| __/ __|
//    | |  __/\__ \ |_\__ \
//    |_|\___||___/\__|___/

#[cfg(test)]
mod tests {
    use crate::AudioBufferStats;
    use crate::direct::SequentialSlice;

    #[test]
    fn stats_integer() {
        let data = vec![1_i32, -1, 1, -1, 1, -1, 1, -1];
        let buffer = SequentialSlice::new(&data, 2, 4).unwrap();
        assert_eq!(buffer.channel_rms(0).unwrap(), 1.0);
        assert_eq!(buffer.channel_peak_to_peak(0).unwrap(), 2);
    }

    #[test]
    fn stats_float() {
        let data = vec![1.0_f32, -1.0, 1.0, -1.0, 1.0, -1.0, 1.0, -1.0];
        let buffer = SequentialSlice::new(&data, 2, 4).unwrap();
        assert_eq!(buffer.channel_rms(0).unwrap(), 1.0);
        assert_eq!(buffer.channel_peak_to_peak(0).unwrap(), 2.0);
    }
}

