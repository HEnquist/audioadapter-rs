use crate::Adapter;

// -------------------- Iterators returning immutable samples --------------------

/// A trait providing convenient iteration through frames and/or channels
/// of an [Adapter].
pub trait AdapterIterators<T> {
    /// Get an iterator that yields the sample value of the specified channel.
    fn iter_channel(&self, channel: usize) -> Option<ChannelSamples<'_, T>>;

    /// Get an iterator that yields iterators for the channels.
    fn iter_channels(&self) -> Channels<'_, T>;

    /// Get an iterator that yields the sample values of the specified frame.
    fn iter_frame(&self, frame: usize) -> Option<FrameSamples<'_, T>>;

    /// Get an iterator that yields iterators for the frames.
    fn iter_frames(&self) -> Frames<'_, T>;
}

impl<T, U> AdapterIterators<T> for U
where
    T: Clone,
    U: Adapter<T>,
{
    fn iter_channel(&self, channel: usize) -> Option<ChannelSamples<'_, T>> {
        ChannelSamples::new(self, channel)
    }

    fn iter_channels(&self) -> Channels<'_, T> {
        Channels::new(self)
    }

    fn iter_frame(&self, frame: usize) -> Option<FrameSamples<'_, T>> {
        FrameSamples::new(self, frame)
    }

    fn iter_frames(&self) -> Frames<'_, T> {
        Frames::new(self)
    }
}

/// An iterator that yields the sample values of a channel.
pub struct ChannelSamples<'b, T> {
    buf: &'b dyn Adapter<T>,
    frame: usize,
    nbr_frames: usize,
    channel: usize,
}

impl<'b, T> ChannelSamples<'b, T>
where
    T: Clone,
{
    pub fn new(buffer: &'b dyn Adapter<T>, channel: usize) -> Option<ChannelSamples<'b, T>> {
        if channel >= buffer.channels() {
            return None;
        }
        let nbr_frames = buffer.frames();
        Some(ChannelSamples {
            buf: buffer as &'b dyn Adapter<T>,
            frame: 0,
            nbr_frames,
            channel,
        })
    }
}

impl<T> Iterator for ChannelSamples<'_, T>
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

/// An iterator that yields the samples values of a frame.
pub struct FrameSamples<'b, T> {
    buf: &'b dyn Adapter<T>,
    frame: usize,
    nbr_channels: usize,
    channel: usize,
}

impl<'b, T> FrameSamples<'b, T>
where
    T: Clone,
{
    pub fn new(buffer: &'b dyn Adapter<T>, frame: usize) -> Option<FrameSamples<'b, T>> {
        if frame >= buffer.frames() {
            return None;
        }
        let nbr_channels = buffer.channels();
        Some(FrameSamples {
            buf: buffer as &'b dyn Adapter<T>,
            channel: 0,
            nbr_channels,
            frame,
        })
    }
}

impl<T> Iterator for FrameSamples<'_, T>
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
pub struct Channels<'b, T> {
    buf: &'b dyn Adapter<T>,
    nbr_channels: usize,
    channel: usize,
}

impl<'b, T> Channels<'b, T>
where
    T: Clone,
{
    pub fn new(buffer: &'b dyn Adapter<T>) -> Channels<'b, T> {
        let nbr_channels = buffer.channels();
        Channels {
            buf: buffer as &'b dyn Adapter<T>,
            channel: 0,
            nbr_channels,
        }
    }
}

impl<'b, T> Iterator for Channels<'b, T>
where
    T: Clone,
{
    type Item = ChannelSamples<'b, T>;

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
pub struct Frames<'b, T> {
    buf: &'b dyn Adapter<T>,
    nbr_frames: usize,
    frame: usize,
}

impl<'b, T> Frames<'b, T>
where
    T: Clone,
{
    pub fn new(buffer: &'b dyn Adapter<T>) -> Frames<'b, T> {
        let nbr_frames = buffer.frames();
        Frames {
            buf: buffer as &'b dyn Adapter<T>,
            frame: 0,
            nbr_frames,
        }
    }
}

impl<'b, T> Iterator for Frames<'b, T>
where
    T: Clone,
{
    type Item = FrameSamples<'b, T>;

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
    extern crate alloc;
    use crate::tests::MinimalAdapter;
    use alloc::vec;

    use super::*;

    #[test]
    fn interleaved() {
        let data = vec![1_i32, 4, 2, 5, 3, 6];
        let buffer = MinimalAdapter::new_from_vec(data, 2, 3);
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
        let data = vec![1_i32, 2, 3, 4, 5, 6];
        let buffer = MinimalAdapter::new_from_vec(data, 2, 3);
        let mut val: i32 = 1;
        for frame in buffer.iter_frames() {
            for sample in frame {
                assert_eq!(sample, val);
                val += 1;
            }
        }
    }
}
