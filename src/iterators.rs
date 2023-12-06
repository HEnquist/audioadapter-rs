use crate::Adapter;

// -------------------- Iterators returning immutable samples --------------------

pub trait AdapterIterators<'a, T: 'a> {
    fn iter_channel(&self, channel: usize) -> Option<ChannelSamples<'a, '_, T>>;

    fn iter_channels(&self) -> Channels<'a, '_, T>;

    fn iter_frame(&self, frame: usize) -> Option<FrameSamples<'a, '_, T>>;

    fn iter_frames(&self) -> Frames<'a, '_, T>;
}

impl<'a, T, U> AdapterIterators<'a, T> for U
where
    T: Clone + 'a,
    U: Adapter<'a, T>,
{
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
}

/// An iterator that yields immutable references to the samples of a channel.
pub struct ChannelSamples<'a, 'b, T> {
    buf: &'b dyn Adapter<'a, T>,
    frame: usize,
    nbr_frames: usize,
    channel: usize,
}

impl<'a, 'b, T> ChannelSamples<'a, 'b, T>
where
    T: Clone,
{
    pub fn new(
        buffer: &'b dyn Adapter<'a, T>,
        channel: usize,
    ) -> Option<ChannelSamples<'a, 'b, T>> {
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

impl<'a, 'b, T> Iterator for ChannelSamples<'a, 'b, T>
where
    T: Clone,
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
pub struct FrameSamples<'a, 'b, T> {
    buf: &'b dyn Adapter<'a, T>,
    frame: usize,
    nbr_channels: usize,
    channel: usize,
}

impl<'a, 'b, T> FrameSamples<'a, 'b, T>
where
    T: Clone,
{
    pub fn new(buffer: &'b dyn Adapter<'a, T>, frame: usize) -> Option<FrameSamples<'a, 'b, T>> {
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

impl<'a, 'b, T> Iterator for FrameSamples<'a, 'b, T>
where
    T: Clone,
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
pub struct Channels<'a, 'b, T> {
    buf: &'b dyn Adapter<'a, T>,
    nbr_channels: usize,
    channel: usize,
}

impl<'a, 'b, T> Channels<'a, 'b, T>
where
    T: Clone,
{
    pub fn new(buffer: &'b dyn Adapter<'a, T>) -> Channels<'a, 'b, T> {
        let nbr_channels = buffer.channels();
        Channels {
            buf: buffer as &'b dyn Adapter<'a, T>,
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

/// An iterator that yields a [FrameSamples] iterator for each frame of an [Adapter].
pub struct Frames<'a, 'b, T> {
    buf: &'b dyn Adapter<'a, T>,
    nbr_frames: usize,
    frame: usize,
}

impl<'a, 'b, T> Frames<'a, 'b, T>
where
    T: Clone,
{
    pub fn new(buffer: &'b dyn Adapter<'a, T>) -> Frames<'a, 'b, T> {
        let nbr_frames = buffer.frames();
        Frames {
            buf: buffer as &'b dyn Adapter<'a, T>,
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

//   _____         _
//  |_   _|__  ___| |_ ___
//    | |/ _ \/ __| __/ __|
//    | |  __/\__ \ |_\__ \
//    |_|\___||___/\__|___/

#[cfg(test)]
mod tests {
    use super::*;
    use crate::direct::{InterleavedSlice, SequentialSlice};

    #[test]
    fn interleaved() {
        let data = [1_i32, 4, 2, 5, 3, 6];
        let buffer = InterleavedSlice::new(&data, 2, 3).unwrap();
        let mut val: i32 = 1;
        for channel in buffer.iter_channels() {
            for sample in channel {
                assert_eq!(sample, val);
                val += 1;
            }
        }
    }

    #[test]
    fn sequential() {
        let data = [1_i32, 3, 5, 2, 4, 6];
        let buffer = SequentialSlice::new(&data, 2, 3).unwrap();
        let mut val: i32 = 1;
        for frame in buffer.iter_frames() {
            for sample in frame {
                assert_eq!(sample, val);
                val += 1;
            }
        }
    }
}
