use crate::{Direct, DirectMut};

// -------------------- Iterators returning immutable samples --------------------

/// An iterator that yields immutable references to the samples of a channel.
pub struct ChannelSamples<'a, 'b, T> {
    buf: &'b dyn Direct<'a, T>,
    frame: usize,
    nbr_frames: usize,
    channel: usize,
}

impl<'a, 'b, T> ChannelSamples<'a, 'b, T>
where
    T: Clone,
{
    pub fn new(buffer: &'b dyn Direct<'a, T>, channel: usize) -> Option<ChannelSamples<'a, 'b, T>> {
        if channel >= buffer.channels() {
            return None;
        }
        let nbr_frames = buffer.frames();
        Some(ChannelSamples {
            buf: buffer as &'b dyn Direct<'a, T>,
            frame: 0,
            nbr_frames,
            channel,
        })
    }
}

impl<'a, 'b, T> Iterator for ChannelSamples<'a, 'b, T>
where
    T: Clone,
{
    type Item = &'b T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.frame >= self.nbr_frames {
            return None;
        }
        let val = unsafe { self.buf.get_sample_unchecked(self.channel, self.frame) };
        self.frame += 1;
        Some(val)
    }
}

/// An iterator that yields immutable references to the samples of a frame.
pub struct FrameSamples<'a, 'b, T> {
    buf: &'b dyn Direct<'a, T>,
    frame: usize,
    nbr_channels: usize,
    channel: usize,
}

impl<'a, 'b, T> FrameSamples<'a, 'b, T>
where
    T: Clone,
{
    pub fn new(buffer: &'b dyn Direct<'a, T>, frame: usize) -> Option<FrameSamples<'a, 'b, T>> {
        if frame >= buffer.frames() {
            return None;
        }
        let nbr_channels = buffer.channels();
        Some(FrameSamples {
            buf: buffer as &'b dyn Direct<'a, T>,
            channel: 0,
            nbr_channels,
            frame,
        })
    }
}

impl<'a, 'b, T> Iterator for FrameSamples<'a, 'b, T>
where
    T: Clone,
{
    type Item = &'b T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.channel >= self.nbr_channels {
            return None;
        }
        let val = unsafe { self.buf.get_sample_unchecked(self.channel, self.frame) };
        self.channel += 1;
        Some(val)
    }
}

// -------------------- Iterators returning immutable iterators --------------------

/// An iterator that yields a [ChannelSamples] iterator for each channel of an [Direct].
pub struct Channels<'a, 'b, T> {
    buf: &'b dyn Direct<'a, T>,
    nbr_channels: usize,
    channel: usize,
}

impl<'a, 'b, T> Channels<'a, 'b, T>
where
    T: Clone,
{
    pub fn new(buffer: &'b dyn Direct<'a, T>) -> Channels<'a, 'b, T> {
        let nbr_channels = buffer.channels();
        Channels {
            buf: buffer as &'b dyn Direct<'a, T>,
            channel: 0,
            nbr_channels,
        }
    }
}

impl<'a, 'b, T> Iterator for Channels<'a, 'b, T>
where
    T: Clone,
{
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

/// An iterator that yields a [FrameSamples] iterator for each frame of an [Direct].
pub struct Frames<'a, 'b, T> {
    buf: &'b dyn Direct<'a, T>,
    nbr_frames: usize,
    frame: usize,
}

impl<'a, 'b, T> Frames<'a, 'b, T>
where
    T: Clone,
{
    pub fn new(buffer: &'b dyn Direct<'a, T>) -> Frames<'a, 'b, T> {
        let nbr_frames = buffer.frames();
        Frames {
            buf: buffer as &'b dyn Direct<'a, T>,
            frame: 0,
            nbr_frames,
        }
    }
}

impl<'a, 'b, T> Iterator for Frames<'a, 'b, T>
where
    T: Clone,
{
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

/// An iterator that yields mutable references to the samples of a channel.
pub struct ChannelSamplesMut<'a, 'b, T> {
    buf: &'b mut dyn DirectMut<'a, T>,
    frame: usize,
    nbr_frames: usize,
    channel: usize,
}

impl<'a, 'b, T> ChannelSamplesMut<'a, 'b, T>
where
    T: Clone,
{
    pub fn new(
        buffer: &'b mut dyn DirectMut<'a, T>,
        channel: usize,
    ) -> Option<ChannelSamplesMut<'a, 'b, T>> {
        if channel >= buffer.channels() {
            return None;
        }
        let nbr_frames = buffer.frames();
        Some(ChannelSamplesMut {
            buf: buffer as &'b mut dyn DirectMut<'a, T>,
            frame: 0,
            nbr_frames,
            channel,
        })
    }
}

impl<'a, 'b, T> Iterator for ChannelSamplesMut<'a, 'b, T>
where
    T: Clone,
{
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.frame >= self.nbr_frames {
            return None;
        }
        let val = unsafe { self.buf.get_sample_unchecked_mut(self.channel, self.frame) };
        // The compiler doesn't know that the iterator never returns the same value twice.
        // Therefore it will not let us return a mutable reference with lifetime 'a.
        // Go via a raw pointer to bypass this.
        let val_ptr = val as *mut T;
        let return_val = unsafe { &mut *val_ptr };
        self.frame += 1;
        Some(return_val)
    }
}

/// An iterator that yields mutable references to the samples of a frame.
pub struct FrameSamplesMut<'a, 'b, T> {
    buf: &'b mut dyn DirectMut<'a, T>,
    frame: usize,
    nbr_channels: usize,
    channel: usize,
}

impl<'a, 'b, T> FrameSamplesMut<'a, 'b, T>
where
    T: Clone,
{
    pub fn new(
        buffer: &'b mut dyn DirectMut<'a, T>,
        frame: usize,
    ) -> Option<FrameSamplesMut<'a, 'b, T>> {
        if frame >= buffer.frames() {
            return None;
        }
        let nbr_channels = buffer.channels();
        Some(FrameSamplesMut {
            buf: buffer as &'b mut dyn DirectMut<'a, T>,
            channel: 0,
            nbr_channels,
            frame,
        })
    }
}

impl<'a, 'b, T> Iterator for FrameSamplesMut<'a, 'b, T>
where
    T: Clone,
{
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.channel >= self.nbr_channels {
            return None;
        }
        let val = unsafe { self.buf.get_sample_unchecked_mut(self.channel, self.frame) };
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

/// An iterator that yields a [ChannelSamplesMut] iterator for each channel of an [Direct].
pub struct ChannelsMut<'a, 'b, T> {
    buf: &'b mut dyn DirectMut<'a, T>,
    nbr_channels: usize,
    channel: usize,
}

impl<'a, 'b, T> ChannelsMut<'a, 'b, T>
where
    T: Clone,
{
    pub fn new(buffer: &'b mut dyn DirectMut<'a, T>) -> ChannelsMut<'a, 'b, T> {
        let nbr_channels = buffer.channels();
        ChannelsMut {
            buf: buffer as &'b mut dyn DirectMut<'a, T>,
            channel: 0,
            nbr_channels,
        }
    }
}

impl<'a, 'b, T> Iterator for ChannelsMut<'a, 'b, T>
where
    T: Clone,
{
    type Item = ChannelSamplesMut<'a, 'b, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.channel >= self.nbr_channels {
            return None;
        }
        // The compiler doesn't know that the iterator never returns the same value twice.
        // Therefore it will not let us return a mutable reference with lifetime 'a.
        // Go via a raw pointer to bypass this.
        let buf_ptr = self.buf as *mut dyn DirectMut<'a, T>;
        let return_buf = unsafe { &mut *buf_ptr };
        let val = ChannelSamplesMut::new(return_buf, self.channel).unwrap();
        self.channel += 1;
        Some(val)
    }
}

/// An iterator that yields a [FrameSamplesMut] iterator for each frame of an [Direct].
pub struct FramesMut<'a, 'b, T> {
    buf: &'b mut dyn DirectMut<'a, T>,
    nbr_frames: usize,
    frame: usize,
}

impl<'a, 'b, T> FramesMut<'a, 'b, T>
where
    T: Clone,
{
    pub fn new(buffer: &'b mut dyn DirectMut<'a, T>) -> FramesMut<'a, 'b, T> {
        let nbr_frames = buffer.frames();
        FramesMut {
            buf: buffer as &'b mut dyn DirectMut<'a, T>,
            frame: 0,
            nbr_frames,
        }
    }
}

impl<'a, 'b, T> Iterator for FramesMut<'a, 'b, T>
where
    T: Clone,
{
    type Item = FrameSamplesMut<'a, 'b, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.frame >= self.nbr_frames {
            return None;
        }
        // The compiler doesn't know that the iterator never returns the same value twice.
        // Therefore it will not let us return a mutable reference with lifetime 'a.
        // Go via a raw pointer to bypass this.
        let buf_ptr = self.buf as *mut dyn DirectMut<'a, T>;
        let return_buf = unsafe { &mut *buf_ptr };
        let val = FrameSamplesMut::new(return_buf, self.frame).unwrap();
        self.frame += 1;
        Some(val)
    }
}
