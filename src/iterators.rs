use crate::traits::Adapter;

// -------------------- Iterators returning immutable samples --------------------

/// An iterator that yields samples of a channel.
pub struct ChannelSamples<'a, 'b, T, U>
where
    U: Adapter<'a, T> + 'a,
{
    buf: U,
    frame: usize,
    nbr_frames: usize,
    channel: usize,
}

impl<'a, 'b, T, U> ChannelSamples<'a, 'b, T, U>
{
    pub fn new(buffer: U, channel: usize) -> Option<ChannelSamples<'a, 'b, T, U>> {
        if channel >= buffer.channels() {
            return None;
        }
        let nbr_frames = buffer.frames();
        Some(ChannelSamples {
            buf: buffer as &'b dyn Adapter<'a, T>,
            frame: 0,
            nbr_frames,
            channel,
        })
    }
}

impl<'a, 'b, T, U> Iterator for ChannelSamples<'a, 'b, T, U>
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.frame >= self.nbr_frames {
            return None;
        }
        let val = unsafe { self.buf.read_sample_unchecked(self.channel, self.frame) };
        self.frame += 1;
        Some(val)
    }
}

/// An iterator that yields immutable references to the samples of a frame.
pub struct FrameSamples<'a, 'b, T, U> 
where
    U: Adapter<'a, T> + 'a,
{
    buf: U,
    frame: usize,
    nbr_channels: usize,
    channel: usize,
}

impl<'a, 'b, T, U> FrameSamples<'a, 'b, T, U>
{
    pub fn new(buffer: U, frame: usize) -> Option<FrameSamples<'a, 'b, T, U>> {
        if frame >= buffer.frames() {
            return None;
        }
        let nbr_channels = buffer.channels();
        Some(FrameSamples {
            buf: buffer as &'b dyn Adapter<'a, T>,
            channel: 0,
            nbr_channels,
            frame,
        })
    }
}

impl<'a, 'b, T, U> Iterator for FrameSamples<'a, 'b, T, U>
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.channel >= self.nbr_channels {
            return None;
        }
        let val = unsafe { self.buf.read_sample_unchecked(self.channel, self.frame) };
        self.channel += 1;
        Some(val)
    }
}

// -------------------- Iterators returning immutable iterators --------------------

/// An iterator that yields a [ChannelSamples] iterator for each channel of an [Adapter].
pub struct Channels<'a, 'b, T, U>
where
    U: Adapter<'a, T> + 'a,
{
    buf: U,
    nbr_channels: usize,
    channel: usize,
}

impl<'a, 'b, T, U> Channels<'a, 'b, T, U>
{
    pub fn new(buffer: U) -> Channels<'a, 'b, T, U> {
        let nbr_channels = buffer.channels();
        Channels {
            buf: buffer,
            channel: 0,
            nbr_channels,
        }
    }
}

impl<'a, 'b, T, U> Iterator for Channels<'a, 'b, T, U>
{
    type Item = ChannelSamples<'a, 'b, T, U>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.channel >= self.nbr_channels {
            return None;
        }
        let val = ChannelSamples::new(self.buf, self.channel).unwrap();
        self.channel += 1;
        Some(val)
    }
}

/// An iterator that yields a [FrameSamples] iterator for each frame of an [Adapter].
pub struct Frames<'a, 'b, T, U>
where
    U: Adapter<'a, T> + 'a,
{
    buf: U,
    nbr_frames: usize,
    frame: usize,
}

impl<'a, 'b, T, U> Frames<'a, 'b, T, U>
{
    pub fn new(buffer: &'b dyn Adapter<'a, T>) -> Frames<'a, 'b, T, U> {
        let nbr_frames = buffer.frames();
        Frames {
            buf: buffer as &'b dyn Adapter<'a, T>,
            frame: 0,
            nbr_frames,
        }
    }
}

impl<'a, 'b, T, U> Iterator for Frames<'a, 'b, T, U>
{
    type Item = FrameSamples<'a, 'b, T, U>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.frame >= self.nbr_frames {
            return None;
        }
        let val = FrameSamples::new(self.buf, self.frame).unwrap();
        self.frame += 1;
        Some(val)
    }
}
