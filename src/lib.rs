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
        if buf.len() < channels {
            return Err(BufferError { desc: format!("Too few channels, {} < {}", buf.len(), channels)});
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

    fn iter_channel(&self, channel: usize) -> Option<ChannelSamples<'a, '_, T>> {
        ChannelSamples::new(self, channel)
    }

    fn iter_channels(&self) -> Channels<'a, '_, T> {
        Channels::new(self)
    }

    fn iter_frame(&self, frame: usize) -> Option<FrameSamples<'a, '_, T>> {
        FrameSamples::new(self, frame)
    }

    fn iter_frames(&self) -> Frames<'a, '_, T> {
        Frames::new(self)
    }

    fn iter_channel_mut(&mut self, channel: usize) -> Option<ChannelSamplesMut<'a, '_, T>> {
        ChannelSamplesMut::new(self, channel)
    }

    fn iter_channels_mut(&mut self) -> ChannelsMut<'a, '_, T> {
        ChannelsMut::new(self)
    }

    fn iter_frame_mut(&mut self, frame: usize) -> Option<FrameSamplesMut<'a, '_, T>> {
        FrameSamplesMut::new(self, frame)
    }

    fn iter_frames_mut(&mut self) -> FramesMut<'a, '_, T> {
        FramesMut::new(self)
    }

    fn read_into_channel_from_slice(&mut self, channel: usize, start: usize, slice: &[T]) -> usize {
        if channel >= self.channels || start >= self.frames {
            return 0;
        }
        let frames_to_read = if (self.frames - start) < slice.len() {
            self.frames - start
        }
        else {
            slice.len()
        };
        self.buf[channel][start..start+frames_to_read].clone_from_slice(&slice[..frames_to_read]);
        frames_to_read
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
            return Err(BufferError { desc: format!("Too few frames, {} < {}", buf.len(), frames)});
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

    fn iter_channel(&self, channel: usize) -> Option<ChannelSamples<'a, '_, T>> {
        ChannelSamples::new(self, channel)
    }

    fn iter_channels(&self) -> Channels<'a, '_, T> {
        Channels::new(self)
    }

    fn iter_frame(&self, frame: usize) -> Option<FrameSamples<'a, '_, T>> {
        FrameSamples::new(self, frame)
    }

    fn iter_frames(&self) -> Frames<'a, '_, T> {
        Frames::new(self)
    }

    fn iter_channel_mut(&mut self, channel: usize) -> Option<ChannelSamplesMut<'a, '_, T>> {
        ChannelSamplesMut::new(self, channel)
    }

    fn iter_channels_mut(&mut self) -> ChannelsMut<'a, '_, T> {
        ChannelsMut::new(self)
    }

    fn iter_frame_mut(&mut self, frame: usize) -> Option<FrameSamplesMut<'a, '_, T>> {
        FrameSamplesMut::new(self, frame)
    }

    fn iter_frames_mut(&mut self) -> FramesMut<'a, '_, T> {
        FramesMut::new(self)
    }

    fn read_into_channel_from_slice(&mut self, channel: usize, start: usize, slice: &[T]) -> usize {
        if channel >= self.channels || start >= self.frames {
            return 0;
        }
        let frames_to_read = if (self.frames - start) < slice.len() {
            self.frames - start
        }
        else {
            slice.len()
        };
        for n in 0..frames_to_read {
            unsafe { *self.get_unchecked_mut(channel, start+n) = slice[n].clone() };
        }
        frames_to_read
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

    fn iter_channel(&self, channel: usize) -> Option<ChannelSamples<'a, '_, T>> {
        ChannelSamples::new(self, channel)
    }

    fn iter_channels(&self) -> Channels<'a, '_, T> {
        Channels::new(self)
    }

    fn iter_frame(&self, frame: usize) -> Option<FrameSamples<'a, '_, T>> {
        FrameSamples::new(self, frame)
    }

    fn iter_frames(&self) -> Frames<'a, '_, T> {
        Frames::new(self)
    }

    fn iter_channel_mut(&mut self, channel: usize) -> Option<ChannelSamplesMut<'a, '_, T>> {
        ChannelSamplesMut::new(self, channel)
    }

    fn iter_channels_mut(&mut self) -> ChannelsMut<'a, '_, T> {
        ChannelsMut::new(self)
    }
    fn iter_frame_mut(&mut self, frame: usize) -> Option<FrameSamplesMut<'a, '_, T>> {
        FrameSamplesMut::new(self, frame)
    }

    fn iter_frames_mut(&mut self) -> FramesMut<'a, '_, T> {
        FramesMut::new(self)
    }

    fn read_into_channel_from_slice(&mut self, channel: usize, start: usize, slice: &[T]) -> usize {
        if channel >= self.channels || start >= self.frames {
            return 0;
        }
        let frames_to_read = if (self.frames - start) < slice.len() {
            self.frames - start
        }
        else {
            slice.len()
        };
        for n in 0..frames_to_read {
            unsafe { *self.get_unchecked_mut(channel, start+n) = slice[n].clone() };
        }
        frames_to_read
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

    fn iter_channel(&self, channel: usize) -> Option<ChannelSamples<'a, '_, T>> {
        ChannelSamples::new(self, channel)
    }

    fn iter_channels(&self) -> Channels<'a, '_, T> {
        Channels::new(self)
    }

    fn iter_frame(&self, frame: usize) -> Option<FrameSamples<'a, '_, T>> {
        FrameSamples::new(self, frame)
    }

    fn iter_frames(&self) -> Frames<'a, '_, T> {
        Frames::new(self)
    }

    fn iter_channel_mut(&mut self, channel: usize) -> Option<ChannelSamplesMut<'a, '_, T>> {
        ChannelSamplesMut::new(self, channel)
    }

    fn iter_channels_mut(&mut self) -> ChannelsMut<'a, '_, T> {
        ChannelsMut::new(self)
    }

    fn iter_frame_mut(&mut self, frame: usize) -> Option<FrameSamplesMut<'a, '_, T>> {
        FrameSamplesMut::new(self, frame)
    }

    fn iter_frames_mut(&mut self) -> FramesMut<'a, '_, T> {
        FramesMut::new(self)
    }

    fn read_into_channel_from_slice(&mut self, channel: usize, start: usize, slice: &[T]) -> usize {
        if channel >= self.channels || start >= self.frames {
            return 0;
        }
        let frames_to_read = if (self.frames - start) < slice.len() {
            self.frames - start
        }
        else {
            slice.len()
        };
        let buffer_start = channel*self.frames + start;
        self.buf[buffer_start..buffer_start+frames_to_read].clone_from_slice(&slice[..frames_to_read]);
        frames_to_read
    }
}


// -------------------- The main buffer trait --------------------

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

    // Useful enough to implement?
    //fn frame_as_slice_available(&'a self) -> bool;
    //fn channel_as_slice_available(&'a self) -> bool;
    //fn frame_as_slice(&'a self, frame: usize) -> Option<&[T]>;
    //fn channel_as_slice(&'a self, channel: usize) -> Option<&[T]>;

    // Convenience methods to read and write to/from slices.
    // Reads/writes the entire slice.
    // Returns an error if self isn't large enough, lengh < (slice length + start)
    // Can use efficient clone_from_slice when the alignments match. 
    fn read_into_channel_from_slice(&mut self, channel: usize, start: usize, slice: &[T]) -> usize; 
    //fn write_from_channel_to_slice(channel: usize, start: usize, slice: &mut [T]) -> Result
    //fn copy_frame_from_slice(channel: usize, start: usize, slice: &[T]) -> Result
    //fn copy_frame_to_slice(channel: usize, start: usize, slice: &mut [T]) -> Result

    fn iter_channel(&self, channel: usize) -> Option<ChannelSamples<'a, '_, T>>;

    fn iter_channels(&self) -> Channels<'a, '_, T>;

    fn iter_frame(&self, frame: usize) -> Option<FrameSamples<'a, '_, T>>;

    fn iter_frames(&self) -> Frames<'a, '_, T>;

    fn iter_channel_mut(&mut self, channel: usize) -> Option<ChannelSamplesMut<'a, '_, T>>;

    fn iter_channels_mut(&mut self) -> ChannelsMut<'a, '_, T>;

    fn iter_frame_mut(&mut self, frame: usize) -> Option<FrameSamplesMut<'a, '_, T>>;

    fn iter_frames_mut(&mut self) -> FramesMut<'a, '_, T>;

}


// -------------------- Iterators returning immutable samples --------------------
pub struct ChannelSamples<'a, 'b, T >{
    buf: &'b dyn AudioBuffer<'a, T>,
    frame: usize,
    nbr_frames: usize,
    channel: usize
}

impl<'a, 'b, T> ChannelSamples<'a, 'b, T> where T: Clone {
    pub fn new(buffer: &'b dyn AudioBuffer<'a, T>, channel: usize) -> Option<ChannelSamples<'a, 'b, T>> {
        if channel >= buffer.channels() {
            return None;
        }
        let nbr_frames = buffer.frames();
        Some(ChannelSamples { buf: buffer as &'b dyn AudioBuffer<'a, T>, frame: 0, nbr_frames, channel })
    }
} 

impl<'a, 'b, T> Iterator for ChannelSamples<'a, 'b, T> where T: Clone {
    type Item = &'b T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.frame >= self.nbr_frames {
            return None;
        }
        let val = unsafe { self.buf.get_unchecked(self.channel, self.frame) };
        self.frame += 1;
        Some(val)
    }
}

pub struct FrameSamples<'a, 'b, T >{
    buf: &'b dyn AudioBuffer<'a, T>,
    frame: usize,
    nbr_channels: usize,
    channel: usize
}

impl<'a, 'b, T> FrameSamples<'a, 'b, T> where T: Clone {
    pub fn new(buffer: &'b dyn AudioBuffer<'a, T>, frame: usize) -> Option<FrameSamples<'a, 'b, T>> {
        if frame >= buffer.frames() {
            return None;
        }
        let nbr_channels = buffer.channels();
        Some(FrameSamples { buf: buffer as &'b dyn AudioBuffer<'a, T>, channel: 0, nbr_channels, frame })
    }
} 

impl<'a, 'b, T> Iterator for FrameSamples<'a, 'b, T> where T: Clone {
    type Item = &'b T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.channel >= self.nbr_channels {
            return None;
        }
        let val = unsafe { self.buf.get_unchecked(self.channel, self.frame) };
        self.channel += 1;
        Some(val)
    }
}

// -------------------- Iterators returning immutable iterators --------------------
pub struct Channels<'a, 'b, T >{
    buf: &'b dyn AudioBuffer<'a, T>,
    nbr_channels: usize,
    channel: usize
}

impl<'a, 'b, T> Channels<'a, 'b, T> where T: Clone {
    pub fn new(buffer: &'b dyn AudioBuffer<'a, T>) -> Channels<'a, 'b, T> {
        let nbr_channels = buffer.channels();
        Channels { buf: buffer as &'b dyn AudioBuffer<'a, T>, channel: 0, nbr_channels }
    }
} 

impl<'a, 'b, T> Iterator for Channels<'a, 'b, T> where T: Clone {
    type Item = ChannelSamples<'a, 'b, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.channel >= self.nbr_channels {
            return None;
        }
        let val = ChannelSamples::new(self.buf, self.channel).unwrap();
        self.channel += 1;
        Some(val)
    }
}

pub struct Frames<'a, 'b, T >{
    buf: &'b dyn AudioBuffer<'a, T>,
    nbr_frames: usize,
    frame: usize
}

impl<'a, 'b, T> Frames<'a, 'b, T> where T: Clone {
    pub fn new(buffer: &'b dyn AudioBuffer<'a, T>) -> Frames<'a, 'b, T> {
        let nbr_frames = buffer.frames();
        Frames { buf: buffer as &'b dyn AudioBuffer<'a, T>, frame: 0, nbr_frames }
    }
} 

impl<'a, 'b, T> Iterator for Frames<'a, 'b, T> where T: Clone {
    type Item = FrameSamples<'a, 'b, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.frame >= self.nbr_frames {
            return None;
        }
        let val = FrameSamples::new(self.buf, self.frame).unwrap();
        self.frame += 1;
        Some(val)
    }
}

// -------------------- Iterators returning mutable samples --------------------

pub struct ChannelSamplesMut<'a, 'b, T >{
    buf: &'b mut dyn AudioBuffer<'a, T>,
    frame: usize,
    nbr_frames: usize,
    channel: usize
}

impl<'a, 'b, T> ChannelSamplesMut<'a, 'b, T> where T: Clone {
    pub fn new(buffer: &'b mut dyn AudioBuffer<'a, T>, channel: usize) -> Option<ChannelSamplesMut<'a, 'b, T>> {
        if channel >= buffer.channels() {
            return None;
        }
        let nbr_frames = buffer.frames();
        Some(ChannelSamplesMut { buf: buffer as &'b mut dyn AudioBuffer<'a, T>, frame: 0, nbr_frames, channel })
    }
} 

impl<'a, 'b, T> Iterator for ChannelSamplesMut<'a, 'b, T> where T: Clone {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.frame >= self.nbr_frames {
            return None;
        }
        let val = unsafe { self.buf.get_unchecked_mut(self.channel, self.frame) };
        // The compiler doesn't know that the iterator never returns the same value twice.
        // Therefore it will not let us return a mutable reference with lifetime 'a.
        // Go via a raw pointer to bypass this.
        let val_ptr = val as *mut T;
        let return_val = unsafe { &mut *val_ptr };
        self.frame += 1;
        Some(return_val) 
    }
}

pub struct FrameSamplesMut<'a, 'b, T >{
    buf: &'b mut dyn AudioBuffer<'a, T>,
    frame: usize,
    nbr_channels: usize,
    channel: usize
}

impl<'a, 'b, T> FrameSamplesMut<'a, 'b, T> where T: Clone {
    pub fn new(buffer: &'b mut dyn AudioBuffer<'a, T>, frame: usize) -> Option<FrameSamplesMut<'a, 'b, T>> {
        if frame >= buffer.frames() {
            return None;
        }
        let nbr_channels = buffer.channels();
        Some(FrameSamplesMut { buf: buffer as &'b mut dyn AudioBuffer<'a, T>, channel: 0, nbr_channels, frame })
    }
} 

impl<'a, 'b, T> Iterator for FrameSamplesMut<'a, 'b, T> where T: Clone {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.channel >= self.nbr_channels {
            return None;
        }
        let val = unsafe { self.buf.get_unchecked_mut(self.channel, self.frame) };
        // The compiler doesn't know that the iterator never returns the same value twice.
        // Therefore it will not let us return a mutable reference with lifetime 'a.
        // Go via a raw pointer to bypass this.
        let val_ptr = val as *mut T;
        let return_val = unsafe { &mut *val_ptr };
        self.channel += 1;
        Some(return_val)
    }
}

// -------------------- Iterators returning mutable iterators --------------------

pub struct ChannelsMut<'a, 'b, T >{
    buf: &'b mut dyn AudioBuffer<'a, T>,
    nbr_channels: usize,
    channel: usize
}

impl<'a, 'b, T> ChannelsMut<'a, 'b, T> where T: Clone {
    pub fn new(buffer: &'b mut dyn AudioBuffer<'a, T>) -> ChannelsMut<'a, 'b, T> {
        let nbr_channels = buffer.channels();
        ChannelsMut { buf: buffer as &'b mut dyn AudioBuffer<'a, T>, channel: 0, nbr_channels }
    }
} 

impl<'a, 'b, T> Iterator for ChannelsMut<'a, 'b, T> where T: Clone {
    type Item = ChannelSamplesMut<'a, 'b, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.channel >= self.nbr_channels {
            return None;
        }
        // The compiler doesn't know that the iterator never returns the same value twice.
        // Therefore it will not let us return a mutable reference with lifetime 'a.
        // Go via a raw pointer to bypass this.
        let buf_ptr = self.buf as *mut dyn AudioBuffer<'a, T>;
        let return_buf = unsafe { &mut *buf_ptr };
        let val = ChannelSamplesMut::new(return_buf, self.channel).unwrap();
        self.channel += 1;
        Some(val)
    }
}

pub struct FramesMut<'a, 'b, T >{
    buf: &'b mut dyn AudioBuffer<'a, T>,
    nbr_frames: usize,
    frame: usize
}

impl<'a, 'b, T> FramesMut<'a, 'b, T> where T: Clone {
    pub fn new(buffer: &'b mut dyn AudioBuffer<'a, T>) -> FramesMut<'a, 'b, T> {
        let nbr_frames = buffer.frames();
        FramesMut { buf: buffer as &'b mut dyn AudioBuffer<'a, T>, frame: 0, nbr_frames }
    }
} 

impl<'a, 'b, T> Iterator for FramesMut<'a, 'b, T> where T: Clone {
    type Item = FrameSamplesMut<'a, 'b, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.frame >= self.nbr_frames {
            return None;
        }
        // The compiler doesn't know that the iterator never returns the same value twice.
        // Therefore it will not let us return a mutable reference with lifetime 'a.
        // Go via a raw pointer to bypass this.
        let buf_ptr = self.buf as *mut dyn AudioBuffer<'a, T>;
        let return_buf = unsafe { &mut *buf_ptr };
        let val = FrameSamplesMut::new(return_buf, self.frame).unwrap();
        self.frame += 1;
        Some(val)
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

        let mut iter1 = buffer.iter_channel(0).unwrap();
        assert_eq!(iter1.next(), Some(&1));
        assert_eq!(iter1.next(), Some(&2));
        assert_eq!(iter1.next(), Some(&3));
        assert_eq!(iter1.next(), None);

        let mut iter2 = buffer.iter_frame(1).unwrap();
        assert_eq!(iter2.next(), Some(&2));
        assert_eq!(iter2.next(), Some(&5));
        assert_eq!(iter2.next(), None);

        let mut sum = 0;
        for channel in buffer.iter_channels() {
            sum += channel.sum::<i32>();
        }
        assert_eq!(sum, 21);

        for channel in buffer.iter_channels_mut() {
            for sample in channel {
                *sample = 2 * *sample;
            }
        }
        let mut sum = 0;
        for channel in buffer.iter_channels() {
            sum += channel.sum::<i32>();
        }
        assert_eq!(sum, 42);
        *buffer.get_mut(1,1).unwrap() = 8;
        assert_eq!(*buffer.get(1,1).unwrap(), 8);
    }


    #[test]
    fn vec_of_frames() {
        let mut data = vec![vec![1_i32,4], vec![2_i32,5], vec![3,6]];
        let mut buffer = SliceOfFrameVecs::new(&mut data, 2, 3).unwrap();
        assert_eq!(*buffer.get(0,0).unwrap(), 1);
        assert_eq!(*buffer.get(1,2).unwrap(), 6);

        let mut iter1 = buffer.iter_channel(0).unwrap();
        assert_eq!(iter1.next(), Some(&1));
        assert_eq!(iter1.next(), Some(&2));
        assert_eq!(iter1.next(), Some(&3));
        assert_eq!(iter1.next(), None);

        let mut iter2 = buffer.iter_frame(1).unwrap();
        assert_eq!(iter2.next(), Some(&2));
        assert_eq!(iter2.next(), Some(&5));
        assert_eq!(iter2.next(), None);

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

        let mut iter1 = buffer.iter_channel(0).unwrap();
        assert_eq!(iter1.next(), Some(&1));
        assert_eq!(iter1.next(), Some(&2));
        assert_eq!(iter1.next(), Some(&3));
        assert_eq!(iter1.next(), None);

        let mut iter2 = buffer.iter_frame(1).unwrap();
        assert_eq!(iter2.next(), Some(&2));
        assert_eq!(iter2.next(), Some(&5));
        assert_eq!(iter2.next(), None);

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

        let mut iter1 = buffer.iter_channel(0).unwrap();
        assert_eq!(iter1.next(), Some(&1));
        assert_eq!(iter1.next(), Some(&2));
        assert_eq!(iter1.next(), Some(&3));
        assert_eq!(iter1.next(), None);

        let mut iter2 = buffer.iter_frame(1).unwrap();
        assert_eq!(iter2.next(), Some(&2));
        assert_eq!(iter2.next(), Some(&5));
        assert_eq!(iter2.next(), None);

        let mut sum = 0;
        for channel in buffer.iter_channels() {
            sum += channel.sum::<i32>();
        }
        assert_eq!(sum, 21);

        *buffer.get_mut(1,1).unwrap() = 8;
        assert_eq!(*buffer.get(1,1).unwrap(), 8);
    }


    #[test]
    fn sequential_sliceops() {
        let mut data = vec![1_i32, 2, 3, 4, 5, 6];
        let mut buffer = SequentialSlice::new(&mut data, 2, 3).unwrap();
        let other = vec![8,9];
        buffer.read_into_channel_from_slice(0, 1, &other);
        let mut buffer2 = SequentialSlice::new(&mut data, 2, 3).unwrap();
        let mut iter1 = buffer2.iter_channel(0).unwrap();
        assert_eq!(iter1.next(), Some(&1));
        assert_eq!(iter1.next(), Some(&8));
        assert_eq!(iter1.next(), Some(&9));
        assert_eq!(iter1.next(), None);
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
