//! # `audio` crate compatibility
//!
//! This module implements the `audioadapter` traits
//! for `ExactSizeBuf` buffers from the `audio` crate.


//use crate::iterators::{
//    ChannelSamples, ChannelSamplesMut, Channels, ChannelsMut, FrameSamples, FrameSamplesMut,
//    Frames, FramesMut,
//};
//use crate::{implement_iterators, implement_iterators_mut};
use crate::{Direct, DirectMut, Indirect, IndirectMut};

use audio_core::{ExactSizeBuf, Buf, BufMut, Sample, Channel, ChannelMut};

impl<'a, T, U> Indirect<'a, T> for U
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

    unsafe fn read_unchecked(&self, channel: usize, frame: usize) -> T {
        self.get(channel).unwrap().get(frame).unwrap()
    }

    // TODO smarter slice copy
}

impl<'a, T, U> IndirectMut<'a, T> for U
where
    T: Clone + Sample + 'a,
    U: BufMut<Sample = T> + ExactSizeBuf<Sample = T>,
{

    unsafe fn write_unchecked(&mut self, channel: usize, frame: usize, value: &T) -> bool {
        *self.get_mut(channel).unwrap().get_mut(frame).unwrap() = *value;
        false
    }

    // TODO smarter slice copy
}

// TODO implement Direct and DirectMut

//   _____         _
//  |_   _|__  ___| |_ ___
//    | |/ _ \/ __| __/ __|
//    | |  __/\__ \ |_\__ \
//    |_|\___||___/\__|___/

#[cfg(test)]
mod tests {
    use super::*;
    use audio::wrap;

    #[test]
    fn read_indirect() {
        let buf = wrap::interleaved(&[1, 2, 3, 4, 5, 6, 7, 8], 2);
        assert_eq!(unsafe {buf.read_unchecked(0,0)}, 1);
        assert_eq!(unsafe {buf.read_unchecked(1,0)}, 2);
        assert_eq!(unsafe {buf.read_unchecked(0,1)}, 3);
        assert_eq!(unsafe {buf.read_unchecked(1,1)}, 4);
    }

    #[test]
    fn write_indirect() {
        let mut buf = audio::buf::Interleaved::<i32>::with_topology(2, 4);
        unsafe {
            buf.write_unchecked(0, 0, &1);
            buf.write_unchecked(1, 0, &2);
            buf.write_unchecked(0, 1, &3);
            buf.write_unchecked(1, 1, &4);

        }
        assert_eq!(buf.get(0).unwrap().get(0).unwrap(), 1);
        assert_eq!(buf.get(1).unwrap().get(0).unwrap(), 2);
        assert_eq!(buf.get(0).unwrap().get(1).unwrap(), 3);
        assert_eq!(buf.get(1).unwrap().get(1).unwrap(), 4);
    }
}






