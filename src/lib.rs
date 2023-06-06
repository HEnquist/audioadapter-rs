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


pub struct SliceOfChannelVecs<'a, T> {
    buf: &'a mut [Vec<T>],
    frames: usize,
    channels: usize,
}  



impl<'a, T> SliceOfChannelVecs<'a, T> {
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


impl<'a, T> AudioBuffer<'a, T>for SliceOfChannelVecs<'a, T> where T: Clone {

    unsafe fn get_unchecked(&self, channel: usize, frame: usize) -> &T {
        return self.buf.get_unchecked(channel).get_unchecked(frame)
    }

    unsafe fn get_unchecked_mut(&mut self, channel: usize, frame: usize) -> &mut T {
        return self.buf.get_unchecked_mut(channel).get_unchecked_mut(frame)
    }

    fn channels(&self) -> usize {
        self.channels
    }

    fn frames(&self) -> usize{
        self.frames
    }

    fn iter_channel(&'a self, channel: usize) -> Option<ChannelSamples<'a, T>> {
        ChannelSamples::new(self, channel)
    }

    fn iter_channels(&'a self) -> Channels<'a, T> {
        Channels::new(self)
    }

    fn iter_frame(&'a self, frame: usize) -> Option<FrameSamples<'a, T>> {
        FrameSamples::new(self, frame)
    }

    fn iter_frames(&'a self) -> Frames<'a, T> {
        Frames::new(self)
    }

}


pub struct SliceOfFrameVecs<'a, T> {
    buf: &'a mut [Vec<T>],
    frames: usize,
    channels: usize,
}  

impl<'a, T> SliceOfFrameVecs<'a, T> {
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

impl<'a, T> AudioBuffer<'a, T>for SliceOfFrameVecs<'a, T> where T: Clone {

    unsafe fn get_unchecked(&self, channel: usize, frame: usize) -> &T {
        return self.buf.get_unchecked(frame).get_unchecked(channel)
    }

    unsafe fn get_unchecked_mut(&mut self, channel: usize, frame: usize) -> &mut T {
        return self.buf.get_unchecked_mut(frame).get_unchecked_mut(channel)
    }

    fn channels(&self) -> usize {
        self.channels
    }

    fn frames(&self) -> usize{
        self.frames
    }

    fn iter_channel(&'a self, channel: usize) -> Option<ChannelSamples<'a, T>> {
        ChannelSamples::new(self, channel)
    }

    fn iter_channels(&'a self) -> Channels<'a, T> {
        Channels::new(self)
    }

    fn iter_frame(&'a self, frame: usize) -> Option<FrameSamples<'a, T>> {
        FrameSamples::new(self, frame)
    }

    fn iter_frames(&'a self) -> Frames<'a, T> {
        Frames::new(self)
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

    unsafe fn get_unchecked(&self, channel: usize, frame: usize) -> &T {
        return self.buf.get_unchecked(frame*self.channels + channel)
    }

    unsafe fn get_unchecked_mut(&mut self, channel: usize, frame: usize) -> &mut T {
        return self.buf.get_unchecked_mut(frame*self.channels + channel)
    }

    fn channels(&self) -> usize {
        self.channels
    }

    fn frames(&self) -> usize{
        self.frames
    }

    fn iter_channel(&'a self, channel: usize) -> Option<ChannelSamples<'a, T>> {
        ChannelSamples::new(self, channel)
    }

    fn iter_channels(&'a self) -> Channels<'a, T> {
        Channels::new(self)
    }

    fn iter_frame(&'a self, frame: usize) -> Option<FrameSamples<'a, T>> {
        FrameSamples::new(self, frame)
    }

    fn iter_frames(&'a self) -> Frames<'a, T> {
        Frames::new(self)
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

    unsafe fn get_unchecked(&self, channel: usize, frame: usize) -> &T {
        return self.buf.get_unchecked(channel*self.frames + frame)
    }

    unsafe fn get_unchecked_mut(&mut self, channel: usize, frame: usize) -> &mut T {
        return self.buf.get_unchecked_mut(channel*self.frames + frame)
    }

    fn channels(&self) -> usize {
        self.channels
    }

    fn frames(&self) -> usize{
        self.frames
    }

    fn iter_channel(&'a self, channel: usize) -> Option<ChannelSamples<'a, T>> {
        ChannelSamples::new(self, channel)
    }

    fn iter_channels(&'a self) -> Channels<'a, T> {
        Channels::new(self)
    }

    fn iter_frame(&'a self, frame: usize) -> Option<FrameSamples<'a, T>> {
        FrameSamples::new(self, frame)
    }

    fn iter_frames(&'a self) -> Frames<'a, T> {
        Frames::new(self)
    }
}

pub trait AudioBuffer<'a, T: Clone + 'a>{

    unsafe fn get_unchecked(&self, channel: usize, frame: usize) -> &T;

    fn get(&self, channel: usize, frame: usize) -> Option<&T> {
        if channel >= self.channels() || frame >= self.frames() {
            return None;
        }
        Some(unsafe { self.get_unchecked(channel, frame)} )
    }

    unsafe fn get_unchecked_mut(&mut self, channel: usize, frame: usize) -> &mut T;

    fn get_mut(&mut self, channel: usize, frame: usize) -> Option<&mut T> {
        if channel >= self.channels() || frame >= self.frames() {
            return None;
        }
        Some(unsafe { self.get_unchecked_mut(channel, frame)} )
    }

    fn channels(&self) -> usize;

    fn frames(&self) -> usize;

    //fn frame_as_slice_available(&'a self) -> bool;
    //fn channel_as_slice_available(&'a self) -> bool;
    //fn frame_as_slice(&'a self, frame: usize) -> Option<&[T]>;
    //fn channel_as_slice(&'a self, channel: usize) -> Option<&[T]>;

    fn iter_channel(&'a self, channel: usize) -> Option<ChannelSamples<'a, T>>;

    fn iter_channels(&'a self) -> Channels<'a, T>;

    fn iter_frame(&'a self, frame: usize) -> Option<FrameSamples<'a, T>>;

    fn iter_frames(&'a self) -> Frames<'a, T>;
}

pub struct SampleIterator<'a, T >{
    iterator: Box<dyn Iterator<Item = &'a T> + 'a>
}

impl<'a, T> Iterator for SampleIterator<'a, T> where T: Clone {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.iterator.next()
    }
}

pub struct ChannelSamples<'a, T >{
    buf: &'a dyn AudioBuffer<'a, T>,
    frame: usize,
    nbr_frames: usize,
    channel: usize
}

impl<'a, T> ChannelSamples<'a, T> where T: Clone {
    pub fn new(buffer: &'a dyn AudioBuffer<'a, T>, channel: usize) -> Option<ChannelSamples<'a, T>> {
        if channel >= buffer.channels() {
            return None;
        }
        let nbr_frames = buffer.frames();
        Some(ChannelSamples { buf: buffer as &'a dyn AudioBuffer<'a, T>, frame: 0, nbr_frames, channel })
    }
} 

impl<'a, T> Iterator for ChannelSamples<'a, T> where T: Clone {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.frame >= self.nbr_frames {
            return None;
        }
        let val = unsafe { self.buf.get_unchecked(self.channel, self.frame) };
        self.frame += 1;
        Some(val)
    }
}

pub struct Channels<'a, T >{
    buf: &'a dyn AudioBuffer<'a, T>,
    nbr_channels: usize,
    channel: usize
}

impl<'a, T> Channels<'a, T> where T: Clone {
    pub fn new(buffer: &'a dyn AudioBuffer<'a, T>) -> Channels<'a, T> {
        let nbr_channels = buffer.channels();
        Channels { buf: buffer as &'a dyn AudioBuffer<'a, T>, channel: 0, nbr_channels }
    }
} 

impl<'a, T> Iterator for Channels<'a, T> where T: Clone {
    type Item = ChannelSamples<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.channel >= self.nbr_channels {
            return None;
        }
        let val = ChannelSamples::new(self.buf, self.channel).unwrap();
        self.channel += 1;
        Some(val)
    }
}



pub struct FrameSamples<'a, T >{
    buf: &'a dyn AudioBuffer<'a, T>,
    frame: usize,
    nbr_channels: usize,
    channel: usize
}

impl<'a, T> FrameSamples<'a, T> where T: Clone {
    pub fn new(buffer: &'a dyn AudioBuffer<'a, T>, frame: usize) -> Option<FrameSamples<'a, T>> {
        if frame >= buffer.frames() {
            return None;
        }
        let nbr_channels = buffer.channels();
        Some(FrameSamples { buf: buffer as &'a dyn AudioBuffer<'a, T>, channel: 0, nbr_channels, frame })
    }
} 

impl<'a, T> Iterator for FrameSamples<'a, T> where T: Clone {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.channel >= self.nbr_channels {
            return None;
        }
        let val = unsafe { self.buf.get_unchecked(self.channel, self.frame) };
        self.channel += 1;
        Some(val)
    }
}

pub struct Frames<'a, T >{
    buf: &'a dyn AudioBuffer<'a, T>,
    nbr_frames: usize,
    frame: usize
}

impl<'a, T> Frames<'a, T> where T: Clone {
    pub fn new(buffer: &'a dyn AudioBuffer<'a, T>) -> Frames<'a, T> {
        let nbr_frames = buffer.frames();
        Frames { buf: buffer as &'a dyn AudioBuffer<'a, T>, frame: 0, nbr_frames }
    }
} 

impl<'a, T> Iterator for Frames<'a, T> where T: Clone {
    type Item = FrameSamples<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.frame >= self.nbr_frames {
            return None;
        }
        let val = FrameSamples::new(self.buf, self.frame).unwrap();
        self.frame += 1;
        Some(val)
    }
}
/*
pub struct ChannelSamplesMut<'a, T >{
    buf: &'a mut dyn AudioBuffer<'a, T>,
    frame: usize,
    nbr_frames: usize,
    channel: usize
}

impl<'a, T> ChannelSamplesMut<'a, T> where T: Clone {
    pub fn new(buffer: &'a mut dyn AudioBuffer<'a, T>, channel: usize) -> Option<ChannelSamplesMut<'a, T>> {
        if channel >= buffer.iter_channels() {
            return None;
        }
        let nbr_frames = buffer.iter_frames();
        Some(ChannelSamplesMut { buf: buffer as &'a mut dyn AudioBuffer<'a, T>, frame: 0, nbr_frames, channel })
    }
} 

impl<'a, T> Iterator for ChannelSamplesMut<'a, T> where T: Clone {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.frame >= self.nbr_frames {
            return None;
        }
        let val = unsafe { self.buf.get_unchecked_mut(self.channel, self.frame) };
        self.frame += 1;
        Some(val) 
    }
}
*/
pub struct SampleIteratorMut<'a, T >{
    iterator: Box<dyn Iterator<Item = &'a mut T> + 'a>
}

impl<'a, T> Iterator for SampleIteratorMut<'a, T> where T: Clone {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        self.iterator.next()
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vec_of_channels() {
        let mut data = vec![vec![1_i32,2,3], vec![4_i32,5,6]];
        let mut buffer = SliceOfChannelVecs::new(&mut data, 2, 3).unwrap();
        assert_eq!(*buffer.get(0,0).unwrap(), 1);
        assert_eq!(*buffer.get(1,2).unwrap(), 6);
        {
            let mut iter1 = buffer.iter_channel(0).unwrap();
            assert_eq!(iter1.next(), Some(&1));
            assert_eq!(iter1.next(), Some(&2));
            assert_eq!(iter1.next(), Some(&3));
            assert_eq!(iter1.next(), None);

            let mut iter2 = buffer.iter_frame(1).unwrap();
            assert_eq!(iter2.next(), Some(&2));
            assert_eq!(iter2.next(), Some(&5));
            assert_eq!(iter2.next(), None);
        }
        let mut sum = 0;
        for channel in buffer.iter_channels() {
            sum += channel.sum::<i32>();
        }
        assert_eq!(sum, 21);

        *buffer.get_mut(1,1).unwrap() = 8;
        assert_eq!(*buffer.get(1,1).unwrap(), 8);
    }


    #[test]
    fn vec_of_frames() {
        let mut data = vec![vec![1_i32,4], vec![2_i32,5], vec![3,6]];
        let mut buffer = SliceOfFrameVecs::new(&mut data, 2, 3).unwrap();
        assert_eq!(*buffer.get(0,0).unwrap(), 1);
        assert_eq!(*buffer.get(1,2).unwrap(), 6);
        {
            let mut iter1 = buffer.iter_channel(0).unwrap();
            assert_eq!(iter1.next(), Some(&1));
            assert_eq!(iter1.next(), Some(&2));
            assert_eq!(iter1.next(), Some(&3));
            assert_eq!(iter1.next(), None);

            let mut iter2 = buffer.iter_frame(1).unwrap();
            assert_eq!(iter2.next(), Some(&2));
            assert_eq!(iter2.next(), Some(&5));
            assert_eq!(iter2.next(), None);
        }
        let mut sum = 0;
        for channel in buffer.iter_channels() {
            sum += channel.sum::<i32>();
        }
        assert_eq!(sum, 21);
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
            let mut iter1 = buffer.iter_channel(0).unwrap();
            assert_eq!(iter1.next(), Some(&1));
            assert_eq!(iter1.next(), Some(&2));
            assert_eq!(iter1.next(), Some(&3));
            assert_eq!(iter1.next(), None);

            let mut iter2 = buffer.iter_frame(1).unwrap();
            assert_eq!(iter2.next(), Some(&2));
            assert_eq!(iter2.next(), Some(&5));
            assert_eq!(iter2.next(), None);
        }
        let mut sum = 0;
        for channel in buffer.iter_channels() {
            sum += channel.sum::<i32>();
        }
        assert_eq!(sum, 21);

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
            let mut iter1 = buffer.iter_channel(0).unwrap();
            assert_eq!(iter1.next(), Some(&1));
            assert_eq!(iter1.next(), Some(&2));
            assert_eq!(iter1.next(), Some(&3));
            assert_eq!(iter1.next(), None);

            let mut iter2 = buffer.iter_frame(1).unwrap();
            assert_eq!(iter2.next(), Some(&2));
            assert_eq!(iter2.next(), Some(&5));
            assert_eq!(iter2.next(), None);
        }
        let mut sum = 0;
        for channel in buffer.iter_channels() {
            sum += channel.sum::<i32>();
        }
        assert_eq!(sum, 21);

        *buffer.get_mut(1,1).unwrap() = 8;
        assert_eq!(*buffer.get(1,1).unwrap(), 8);
    }

    #[test]
    fn iterator() {
        let mut data = vec![1_i32, 2, 3, 4, 5, 6];
        let buffer = SequentialSlice::new(&mut data, 2, 3).unwrap();
        let mut iter1 = ChannelSamples::new(&buffer as &dyn AudioBuffer<'_, i32>, 0).unwrap();
        assert_eq!(iter1.next(), Some(&1));
        assert_eq!(iter1.next(), Some(&2));
        assert_eq!(iter1.next(), Some(&3));
        assert_eq!(iter1.next(), None);

    }

    // This tests that an AudioBuffer is object safe.
    #[test]
    fn boxed_buffer() {
        let mut data = vec![1_i32, 2, 3, 4, 5, 6];
        let boxed: Box<dyn AudioBuffer<i32>> = Box::new(SequentialSlice::new(&mut data, 2, 3).unwrap());
        assert_eq!(*boxed.get(0,0).unwrap(), 1);
    }

}  
