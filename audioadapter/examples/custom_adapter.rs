// This example shows how to implement the
// `Adapter` trait for a custom struct.
// The data here is a vector of strings,
// that get converted to numbers on reading.
// `Adapter` is an unsafe trait, so implementations must guarantee
// that `channels()` and `frames()` always describe valid bounds
// for unchecked sample access while the adapter is in use.

use audioadapter::Adapter;
use num_traits::Zero;
use std::str::FromStr;

struct MyStruct<T> {
    // `MyStruct` produces `T` from strings on read but never stores one, so use
    // a `fn() -> T` marker rather than `PhantomData<T>` to avoid implying ownership.
    _phantom: core::marker::PhantomData<fn() -> T>,
    data: Vec<String>,
    channels: usize,
}

unsafe impl<T> Adapter<T> for MyStruct<T>
where
    T: Clone + FromStr + Zero,
{
    fn channels(&self) -> usize {
        self.channels
    }

    fn frames(&self) -> usize {
        self.data.len() / self.channels
    }

    unsafe fn read_sample_unchecked(&self, channel: usize, frame: usize) -> T {
        let raw = unsafe { self.data.get_unchecked(self.channels * frame + channel) };
        raw.parse::<T>().unwrap_or(T::zero())
    }
}

fn main() {
    let data = vec![
        "1".to_owned(),
        "2".to_owned(),
        "3".to_owned(),
        "4".to_owned(),
        "5".to_owned(),
        "6".to_owned(),
    ];
    let adapter: MyStruct<f32> = MyStruct {
        _phantom: core::marker::PhantomData,
        data,
        channels: 2,
    };
    for channel in 0..adapter.channels() {
        for frame in 0..adapter.frames() {
            let value = adapter.read_sample(channel, frame).unwrap();
            println!("Channel: {}, frame: {}, value: {}", channel, frame, value);
        }
    }
}
