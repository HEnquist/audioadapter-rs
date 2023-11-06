// This example shows a way to read 16-bit integer samples from raw bytes.
// The conversion is done by first creating a view of the raw bytes as
// a slice of 2-byte arrays, &[[u8; 2]], and wrapping this view
// as an Adapter.
// The samples values are then read using the Adapter trait methods
// and converted to i16 with i16::from_le_bytes().

use audioadapter::direct::InterleavedSlice;
use audioadapter::Adapter;

fn main() {
    let channels = 2;
    let frames = 10;

    // Create some nonsense data.
    // Let's imagine that the data comes from a typical stereo .wav file,
    // meaning that the samples are 16 bit little-endian signed integers,
    // with two channels stored in interleaved order.
    // Every byte is set to the value 0x1.
    // When combining them two by two to make 16-bit integers,
    // the dummy sample values become 0x11 = 257.
    let mut byte_data = vec![1_u8; channels * frames * 2];

    // Create a view of the data with as a slice of [u8; 2]
    let data_view = unsafe {
        let ptr = byte_data.as_mut_ptr() as *mut [u8; 2];
        let len = byte_data.len();
        std::slice::from_raw_parts_mut(ptr, len / 2)
    };

    // Create an Adapter for the [u8; 2] view
    let buffer = InterleavedSlice::new(data_view, channels, frames).unwrap();

    // Loop over all samples and print their values
    for channel in 0..buffer.channels() {
        for frame in 0..buffer.frames() {
            let bytes = buffer.read_sample(channel, frame).unwrap();
            let value = i16::from_le_bytes(bytes);
            println!("Channel: {}, frame: {}, value: {}", channel, frame, value);
        }
    }
}
