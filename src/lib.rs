use std::error;
use std::fmt;
use std::slice;
use std::iter::{StepBy, Skip, Take};

#[derive(Debug)]
pub struct BufferError {
    desc: String,
}

impl fmt::Display for BufferError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.desc)
    }
}

impl error::Error for BufferError {
    fn description(&self) -> &str {
        &self.desc
    }
}

impl BufferError {
    pub fn new(desc: &str) -> Self {
        BufferError {
            desc: desc.to_owned(),
        }
    }
}


pub struct VecOfChannels<'a, T> {
    buf: &'a mut [Vec<T>],
    frames: usize,
    channels: usize,
}  



impl<'a, T> VecOfChannels<'a, T> {
    pub fn new(buf: &'a mut [Vec<T>], channels: usize, frames: usize) -> Result<Self, BufferError> {
        if buf.len() != channels {
            return Err(BufferError { desc: format!("Bad number of channels, {} != {}", buf.len(), channels)});
        }
        for (idx, chan) in buf.iter().enumerate() {
            if chan.len() < frames {
                return Err(BufferError { desc: format!("Channel {} is too short, {} < {}", idx, chan.len(), frames)});
            }
        }
        Ok(Self {
            buf,
            frames,
            channels,
        })
    }
}

impl<'a, T> AudioBuffer<'a, T>for VecOfChannels<'a, T> where T: Clone {

    type ChannelIterator = Take<slice::Iter<'a, T>>;
    type FrameIterator = Box<dyn Iterator<Item=&'a T> + 'a>;

    fn get(&self, channel: usize, frame: usize) -> Option<&T> {
        return self.buf.get(channel).and_then(|ch| ch.get(frame))
    }

    fn get_mut(&mut self, channel: usize, frame: usize) -> Option<&mut T> {
        return self.buf.get_mut(channel).and_then(|ch| ch.get_mut(frame))
    }

    fn channels(&self) -> usize {
        self.channels
    }

    fn frames(&self) -> usize{
        self.frames
    }

    fn channel(&'a self, channel: usize) -> Self::ChannelIterator {
        self.buf[channel].iter().take(self.frames)
    }

    fn frame(&'a self, frame: usize) -> Self::FrameIterator {
        Box::new(self.buf.iter().take(self.channels).map(move |v| &v[frame]))
    }

}


pub struct VecOfFrames<'a, T> {
    buf: &'a mut [Vec<T>],
    frames: usize,
    channels: usize,
}  

impl<'a, T> VecOfFrames<'a, T> {
    pub fn new(buf: &'a mut [Vec<T>], channels: usize, frames: usize) -> Result<Self, BufferError> {
        if buf.len() < frames {
            return Err(BufferError { desc: format!("Bad number of frames, {} != {}", buf.len(), frames)});
        }
        for (idx, frame) in buf.iter().enumerate() {
            if frame.len() < channels {
                return Err(BufferError { desc: format!("Frame {} is too short, {} < {}", idx, frame.len(), channels)});
            }
        }
        Ok(Self {
            buf,
            frames,
            channels,
        })
    }
}

impl<'a, T> AudioBuffer<'a, T>for VecOfFrames<'a, T> where T: Clone {
    type ChannelIterator = Box<dyn Iterator<Item=&'a T> + 'a>;
    type FrameIterator = Take<slice::Iter<'a, T>>;

    fn get(&self, channel: usize, frame: usize) -> Option<&T> {
        return self.buf.get(frame).and_then(|ch| ch.get(channel))
    }

    fn get_mut(&mut self, channel: usize, frame: usize) -> Option<&mut T> {
        return self.buf.get_mut(frame).and_then(|ch| ch.get_mut(channel))
    }

    fn channels(&self) -> usize {
        self.channels
    }

    fn frames(&self) -> usize{
        self.frames
    }

    fn channel(&'a self, channel: usize) -> Self::ChannelIterator {
        Box::new(self.buf.iter().take(self.frames).map(move |v| &v[channel]))
    }

    fn frame(&'a self, frame: usize) -> Self::FrameIterator {
        self.buf[frame].iter().take(self.channels)

    }
}

pub struct InterleavedSlice<'a, T> {
    buf: &'a mut [T],
    frames: usize,
    channels: usize,
}  

impl<'a, T> InterleavedSlice<'a, T> {
    pub fn new(buf: &'a mut [T], channels: usize, frames: usize) -> Result<Self, BufferError> {
        if buf.len() < frames*channels {
            return Err(BufferError { desc: format!("Buffer is too short, {} < {}", buf.len(), frames*channels)});
        }
        Ok(Self {
            buf,
            frames,
            channels,
        })
    }
}

impl<'a, T> AudioBuffer<'a, T>for InterleavedSlice<'a, T> where T: Clone {
    type ChannelIterator = Take<StepBy<Skip<slice::Iter<'a, T>>>>;
    type FrameIterator = Take<Skip<slice::Iter<'a, T>>>;

    fn get(&self, channel: usize, frame: usize) -> Option<&T> {
        return self.buf.get(frame*self.channels + channel)
    }

    fn get_mut(&mut self, channel: usize, frame: usize) -> Option<&mut T> {
        return self.buf.get_mut(frame*self.channels + channel)
    }

    fn channels(&self) -> usize {
        self.channels
    }

    fn frames(&self) -> usize{
        self.frames
    }

    fn channel(&'a self, channel: usize) -> Self::ChannelIterator {
        self.buf.iter().skip(channel).step_by(self.channels).take(self.frames)
    }

    fn frame(&'a self, frame: usize) -> Self::FrameIterator {
        self.buf.iter().skip(frame * self.channels).take(self.channels)
    }
}

pub struct SequentialSlice<'a, T> {
    buf: &'a mut [T],
    frames: usize,
    channels: usize,
}  

impl<'a, T> SequentialSlice<'a, T> {
    pub fn new(buf: &'a mut [T], channels: usize, frames: usize) -> Result<Self, BufferError> {
        if buf.len() < frames*channels {
            return Err(BufferError { desc: format!("Buffer is too short, {} < {}", buf.len(), frames*channels)});
        }
        Ok(Self {
            buf,
            frames,
            channels,
        })
    }
}

impl<'a, T> AudioBuffer<'a, T>for SequentialSlice<'a, T> where T: Clone {
    type ChannelIterator = Take<Skip<slice::Iter<'a, T>>>;
    type FrameIterator = Take<StepBy<Skip<slice::Iter<'a, T>>>>;

    fn get(&self, channel: usize, frame: usize) -> Option<&T> {
        return self.buf.get(channel*self.frames + frame)
    }

    fn get_mut(&mut self, channel: usize, frame: usize) -> Option<&mut T> {
        return self.buf.get_mut(channel*self.frames + frame)
    }

    fn channels(&self) -> usize {
        self.channels
    }

    fn frames(&self) -> usize{
        self.frames
    }

    fn channel(&'a self, channel: usize) -> Self::ChannelIterator {
        self.buf.iter().skip(self.frames * channel).take(self.frames)
    }

    fn frame(&'a self, frame: usize) -> Self::FrameIterator {
        self.buf.iter().skip(frame).step_by(self.frames).take(self.channels)
    }

}

pub trait AudioBuffer<'a, T: Clone + 'a> {

    type ChannelIterator: Iterator<Item=&'a T>;
    type FrameIterator: Iterator<Item=&'a T>;

    fn get(&self, channel: usize, frame: usize) -> Option<&T>;

    fn get_mut(&mut self, channel: usize, frame: usize) -> Option<&mut T>;

    fn channels(&self) -> usize;

    fn frames(&self) -> usize;

    fn channel(&'a self, channel: usize) -> Self::ChannelIterator;

    fn frame(&'a self, frame: usize) -> Self::FrameIterator;
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vec_of_channels() {
        let mut data = vec![vec![1_i32,2,3], vec![4_i32,5,6]];
        let mut buffer = VecOfChannels::new(&mut data, 2, 3).unwrap();
        assert_eq!(*buffer.get(0,0).unwrap(), 1);
        assert_eq!(*buffer.get(1,2).unwrap(), 6);
        {
            let mut iter1 = buffer.channel(0);
            assert_eq!(iter1.next(), Some(&1));
            assert_eq!(iter1.next(), Some(&2));
            assert_eq!(iter1.next(), Some(&3));
            assert_eq!(iter1.next(), None);

            let mut iter2 = buffer.frame(1);
            assert_eq!(iter2.next(), Some(&2));
            assert_eq!(iter2.next(), Some(&5));
            assert_eq!(iter2.next(), None);
        }

        *buffer.get_mut(1,1).unwrap() = 8;
        assert_eq!(*buffer.get(1,1).unwrap(), 8);
    }


    #[test]
    fn vec_of_frames() {
        let mut data = vec![vec![1_i32,4], vec![2_i32,5], vec![3,6]];
        let mut buffer = VecOfFrames::new(&mut data, 2, 3).unwrap();
        assert_eq!(*buffer.get(0,0).unwrap(), 1);
        assert_eq!(*buffer.get(1,2).unwrap(), 6);
        {
            let mut iter1 = buffer.channel(0);
            assert_eq!(iter1.next(), Some(&1));
            assert_eq!(iter1.next(), Some(&2));
            assert_eq!(iter1.next(), Some(&3));
            assert_eq!(iter1.next(), None);

            let mut iter2 = buffer.frame(1);
            assert_eq!(iter2.next(), Some(&2));
            assert_eq!(iter2.next(), Some(&5));
            assert_eq!(iter2.next(), None);
        }
        *buffer.get_mut(1,1).unwrap() = 8;
        assert_eq!(*buffer.get(1,1).unwrap(), 8);
    }

    #[test]
    fn interleaved() {
        let mut data = vec![1_i32, 4, 2, 5, 3, 6];
        let mut buffer = InterleavedSlice::new(&mut data, 2, 3).unwrap();
        assert_eq!(*buffer.get(0,0).unwrap(), 1);
        assert_eq!(*buffer.get(1,2).unwrap(), 6);

        {
            let mut iter1 = buffer.channel(0);
            assert_eq!(iter1.next(), Some(&1));
            assert_eq!(iter1.next(), Some(&2));
            assert_eq!(iter1.next(), Some(&3));
            assert_eq!(iter1.next(), None);

            let mut iter2 = buffer.frame(1);
            assert_eq!(iter2.next(), Some(&2));
            assert_eq!(iter2.next(), Some(&5));
            assert_eq!(iter2.next(), None);
        }

        *buffer.get_mut(1,1).unwrap() = 8;
        assert_eq!(*buffer.get(1,1).unwrap(), 8);
    }

    #[test]
    fn sequential() {
        let mut data = vec![1_i32, 2, 3, 4, 5, 6];
        let mut buffer = SequentialSlice::new(&mut data, 2, 3).unwrap();
        assert_eq!(*buffer.get(0,0).unwrap(), 1);
        assert_eq!(*buffer.get(1,2).unwrap(), 6);
        {
            let mut iter1 = buffer.channel(0);
            assert_eq!(iter1.next(), Some(&1));
            assert_eq!(iter1.next(), Some(&2));
            assert_eq!(iter1.next(), Some(&3));
            assert_eq!(iter1.next(), None);

            let mut iter2 = buffer.frame(1);
            assert_eq!(iter2.next(), Some(&2));
            assert_eq!(iter2.next(), Some(&5));
            assert_eq!(iter2.next(), None);
        }
        *buffer.get_mut(1,1).unwrap() = 8;
        assert_eq!(*buffer.get(1,1).unwrap(), 8);
    }

}  
