//! # `audio` crate compatibility
//!
//! This module implements the `audioadapter` traits
//! for `ExactSizeBuf` buffers from the `audio` crate.

use crate::{Adapter, AdapterMut};

use audio_core::{Buf, BufMut, Channel, ChannelMut, ExactSizeBuf, Sample};

impl<'a, T, U> Adapter<'a, T> for U
where
    T: Clone + Sample + 'a,
    U: Buf<Sample = T> + ExactSizeBuf<Sample = T>,
{
    fn channels(&self) -> usize {
        self.channels()
    }

    fn frames(&self) -> usize {
        self.frames()
    }

    unsafe fn read_sample_unchecked(&self, channel: usize, frame: usize) -> T {
        self.get(channel).unwrap().get(frame).unwrap()
    }

    fn write_from_channel_to_slice(&self, channel: usize, skip: usize, slice: &mut [T]) -> usize {
        if channel >= self.channels() || skip >= self.frames() {
            return 0;
        }
        let frames_to_write = if (self.frames() - skip) < slice.len() {
            self.frames() - skip
        } else {
            slice.len()
        };
        let chan = self.get(channel).unwrap();
        chan.iter()
            .skip(skip)
            .take(frames_to_write)
            .zip(slice.iter_mut())
            .for_each(|(s, o)| *o = s);
        frames_to_write
    }
}

impl<'a, T, U> AdapterMut<'a, T> for U
where
    T: Clone + Sample + 'a,
    U: BufMut<Sample = T> + ExactSizeBuf<Sample = T>,
{
    unsafe fn write_sample_unchecked(&mut self, channel: usize, frame: usize, value: &T) -> bool {
        *self.get_mut(channel).unwrap().get_mut(frame).unwrap() = *value;
        false
    }

    fn write_from_slice_to_channel(
        &mut self,
        channel: usize,
        skip: usize,
        slice: &[T],
    ) -> (usize, usize) {
        if channel >= Adapter::channels(self) || skip >= Adapter::frames(self) {
            return (0, 0);
        }
        let frames_to_read = if (Adapter::frames(self) - skip) < slice.len() {
            Adapter::frames(self) - skip
        } else {
            slice.len()
        };
        let mut chan = self.get_mut(channel).unwrap();
        chan.iter_mut()
            .skip(skip)
            .take(frames_to_read)
            .zip(slice.iter())
            .for_each(|(s, o)| *s = *o);
        (frames_to_read, 0)
    }
}

//   _____         _
//  |_   _|__  ___| |_ ___
//    | |/ _ \/ __| __/ __|
//    | |  __/\__ \ |_\__ \
//    |_|\___||___/\__|___/

#[cfg(test)]
mod tests {
    use super::*;
    use crate::converter::ConvertI16;
    use audio::wrap;

    #[test]
    fn read_indirect() {
        let buf = wrap::interleaved(&[1, 2, 3, 4, 5, 6, 7, 8], 2);
        assert_eq!(unsafe { buf.read_sample_unchecked(0, 0) }, 1);
        assert_eq!(unsafe { buf.read_sample_unchecked(1, 0) }, 2);
        assert_eq!(unsafe { buf.read_sample_unchecked(0, 1) }, 3);
        assert_eq!(unsafe { buf.read_sample_unchecked(1, 1) }, 4);
    }

    #[test]
    fn write_indirect() {
        let mut buf = audio::buf::Interleaved::<i32>::with_topology(2, 4);
        unsafe {
            buf.write_sample_unchecked(0, 0, &1);
            buf.write_sample_unchecked(1, 0, &2);
            buf.write_sample_unchecked(0, 1, &3);
            buf.write_sample_unchecked(1, 1, &4);
        }
        assert_eq!(buf.get(0).unwrap().get(0).unwrap(), 1);
        assert_eq!(buf.get(1).unwrap().get(0).unwrap(), 2);
        assert_eq!(buf.get(0).unwrap().get(1).unwrap(), 3);
        assert_eq!(buf.get(1).unwrap().get(1).unwrap(), 4);
    }

    #[test]
    fn read_to_slice() {
        let mut other = vec![0; 3];
        let buf = wrap::interleaved(&[1, 2, 3, 4, 5, 6, 7, 8], 2);
        buf.write_from_channel_to_slice(0, 1, &mut other);
        assert_eq!(other[0], 3);
        assert_eq!(other[1], 5);
        assert_eq!(other[2], 7);
    }

    #[test]
    fn write_to_slice() {
        let other = vec![1, 2, 3];
        let mut buf = audio::buf::Interleaved::<i32>::with_topology(2, 4);
        buf.write_from_slice_to_channel(0, 1, &other);
        assert_eq!(buf.get(0).unwrap().get(0).unwrap(), 0);
        assert_eq!(buf.get(0).unwrap().get(1).unwrap(), 1);
        assert_eq!(buf.get(0).unwrap().get(2).unwrap(), 2);
        assert_eq!(buf.get(0).unwrap().get(3).unwrap(), 3);
    }

    #[test]
    fn read_direct() {
        let buf = wrap::interleaved(&[1, 2, 3, 4, 5, 6, 7, 8], 2);
        assert_eq!(buf.get_sample(0, 0), Some(&1));
        assert_eq!(buf.get_sample(1, 0), Some(&2));
        assert_eq!(buf.get_sample(0, 1), Some(&3));
        assert_eq!(buf.get_sample(1, 1), Some(&4));
    }

    #[test]
    fn write_direct() {
        let mut buf = audio::buf::Interleaved::<i32>::with_topology(2, 4);
        *buf.get_sample_mut(0, 0).unwrap() = 1;
        *buf.get_sample_mut(1, 0).unwrap() = 2;
        *buf.get_sample_mut(0, 1).unwrap() = 3;
        *buf.get_sample_mut(1, 1).unwrap() = 4;
        assert_eq!(buf.get(0).unwrap().get(0).unwrap(), 1);
        assert_eq!(buf.get(1).unwrap().get(0).unwrap(), 2);
        assert_eq!(buf.get(0).unwrap().get(1).unwrap(), 3);
        assert_eq!(buf.get(1).unwrap().get(1).unwrap(), 4);
    }

    #[test]
    fn test_convert_i16() {
        let data: [i16; 6] = [0, i16::MIN, 1 << 14, -(1 << 14), 1 << 13, -(1 << 13)];
        let buffer = wrap::interleaved(&data, 2);
        let converter: ConvertI16<&dyn Adapter<i16>, f32> =
            ConvertI16::new(&buffer as &dyn Adapter<i16>);
        assert_eq!(converter.read_sample(0, 0).unwrap(), 0.0);
        assert_eq!(converter.read_sample(1, 0).unwrap(), -1.0);
        assert_eq!(converter.read_sample(0, 1).unwrap(), 0.5);
        assert_eq!(converter.read_sample(1, 1).unwrap(), -0.5);
        assert_eq!(converter.read_sample(0, 2).unwrap(), 0.25);
        assert_eq!(converter.read_sample(1, 2).unwrap(), -0.25);
    }
}
