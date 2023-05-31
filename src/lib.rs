use std::error;
use std::fmt;


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

impl<'a, T> AudioBuffer<'a, T>for VecOfChannels<'a, T> where T: Copy {
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

impl<'a, T> AudioBuffer<'a, T>for VecOfFrames<'a, T> where T: Copy {
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
}

pub struct Interleaved<'a, T> {
    buf: &'a mut [T],
    frames: usize,
    channels: usize,
}  

impl<'a, T> Interleaved<'a, T> {
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

impl<'a, T> AudioBuffer<'a, T>for Interleaved<'a, T> where T: Copy {
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
}

pub struct NonInterleaved<'a, T> {
    buf: &'a mut [T],
    frames: usize,
    channels: usize,
}  

impl<'a, T> NonInterleaved<'a, T> {
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

impl<'a, T> AudioBuffer<'a, T>for NonInterleaved<'a, T> where T: Copy {
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
}

pub trait AudioBuffer<'a, T: Copy> {
    fn get(&self, channel: usize, frame: usize) -> Option<&T>;

    fn get_mut(&mut self, channel: usize, frame: usize) -> Option<&mut T>;

    fn channels(&self) -> usize;

    fn frames(&self) -> usize;

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
        *buffer.get_mut(1,1).unwrap() = 8;
        assert_eq!(*buffer.get(1,1).unwrap(), 8);
    }

    #[test]
    fn vec_of_frames() {
        let mut data = vec![vec![1_i32,4], vec![2_i32,5], vec![3,6]];
        let mut buffer = VecOfFrames::new(&mut data, 2, 3).unwrap();
        assert_eq!(*buffer.get(0,0).unwrap(), 1);
        assert_eq!(*buffer.get(1,2).unwrap(), 6);
        *buffer.get_mut(1,1).unwrap() = 8;
        assert_eq!(*buffer.get(1,1).unwrap(), 8);
    }

    #[test]
    fn interleaved() {
        let mut data = vec![1_i32, 4, 2, 5, 3, 6];
        let mut buffer = Interleaved::new(&mut data, 2, 3).unwrap();
        assert_eq!(*buffer.get(0,0).unwrap(), 1);
        assert_eq!(*buffer.get(1,2).unwrap(), 6);
        *buffer.get_mut(1,1).unwrap() = 8;
        assert_eq!(*buffer.get(1,1).unwrap(), 8);
    }

    #[test]
    fn non_interleaved() {
        let mut data = vec![1_i32, 2, 3, 4, 5, 6];
        let mut buffer = NonInterleaved::new(&mut data, 2, 3).unwrap();
        assert_eq!(*buffer.get(0,0).unwrap(), 1);
        assert_eq!(*buffer.get(1,2).unwrap(), 6);
        *buffer.get_mut(1,1).unwrap() = 8;
        assert_eq!(*buffer.get(1,1).unwrap(), 8);
    }

}  
