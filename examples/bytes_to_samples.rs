// This example shows a way to read 16-bit integer samples from raw bytes.
// The conversion is done by first creating a view of the raw bytes as
// a slice of 2-byte arrays, &[[u8; 2]], and wrapping this view
// as an AudioBuffer.
// The samples values are then read using the AudioBuffer trait methods
// and converted to i16 with i16::from_le_bytes().

use audiobuffer::traits::AudioBuffer;
use audiobuffer::direct::InterleavedSlice;

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

    // Create an AudioBuffer for the [u8; 2] view
    let buffer = InterleavedSlice::new(data_view, channels, frames).unwrap();

    // Loop over all samples and print their values
    for (ch_idx, channel) in buffer.iter_channels().enumerate() {
        for (frame_idx, sample_bytes) in channel.enumerate() {
            let value = i16::from_le_bytes(*sample_bytes);
            println!(
                "Channel: {}, frame: {}, value: {}",
                ch_idx, frame_idx, value
            );
        }
    }
}
