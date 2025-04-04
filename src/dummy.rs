//! # Dummy adapter
//!
//! A struct that implement the [Adapter] and [AdapterMut] traits.
//! It returns dummy values on read, and discards data on write.

use crate::implement_size_getters;
use crate::{Adapter, AdapterMut};

/// A dummy adapter that returns a constant value on read,
/// and discards any data written to it.
pub struct Dummy<U> {
    default: U,
    frames: usize,
    channels: usize,
}

impl<T> Dummy<T>
where
    T: Clone,
{
    /// Create a new `Dummy` that always returns `value` on read.
    pub fn new(value: T, channels: usize, frames: usize) -> Self {
        Self {
            default: value,
            frames,
            channels,
        }
    }
}

impl<'a, T> Adapter<'a, T> for Dummy<T>
where
    T: Clone + 'a,
{
    unsafe fn read_sample_unchecked(&self, _channel: usize, _frame: usize) -> T {
        self.default.clone()
    }

    implement_size_getters!();
}

impl<'a, T> AdapterMut<'a, T> for Dummy<T>
where
    T: Clone + 'a,
{
    unsafe fn write_sample_unchecked(
        &mut self,
        _channel: usize,
        _frame: usize,
        _value: &T,
    ) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dummy() {
        let mut dummy = Dummy::<usize>::new(1234, 2, 3);
        for channel in 0..2 {
            for frame in 0..3 {
                assert_eq!(dummy.read_sample(channel, frame), Some(1234));
            }
        }
        assert_eq!(dummy.read_sample(3, 0), None);
        assert_eq!(dummy.read_sample(0, 4), None);
        for channel in 0..2 {
            for frame in 0..3 {
                assert_eq!(dummy.write_sample(channel, frame, &123), Some(false));
            }
        }
        assert_eq!(dummy.write_sample(3, 0, &123), None);
        assert_eq!(dummy.write_sample(0, 4, &123), None);
    }
}
