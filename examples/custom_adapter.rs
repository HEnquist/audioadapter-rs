// This example shows how to implement the
// `Indirect` trait for a custom struct.
// The data here is a vector of strings,
// that get converted to numbers on reading.

use audioadapter::Indirect;
use num_traits::Zero;
use std::str::FromStr;

struct MyStruct<'a, T> {
    _phantom: core::marker::PhantomData<&'a T>,
    data: Vec<String>,
    channels: usize,
}

impl<'a, T> Indirect<'a, T> for MyStruct<'a, T>
where
    T: Clone + FromStr + Zero + 'a,
{
    fn channels(&self) -> usize {
        self.channels
    }

    fn frames(&self) -> usize {
        self.data.len() / self.channels
    }

    unsafe fn read_unchecked(&self, channel: usize, frame: usize) -> T {
        let raw = self.data.get_unchecked(self.channels * frame + channel);
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
    let adapter: MyStruct<'_, f32> = MyStruct {
        _phantom: core::marker::PhantomData,
        data,
        channels: 2,
    };
    for channel in 0..adapter.channels() {
        for frame in 0..adapter.frames() {
            let value = adapter.read(channel, frame).unwrap();
            println!("Channel: {}, frame: {}, value: {}", channel, frame, value);
        }
    }
}
