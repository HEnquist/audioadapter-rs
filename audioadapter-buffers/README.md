# audioadapter-buffers

This crate is part of the `audioadapter` family.

The `audioadapter` family consists of three crates:
- [audioadapter](https://crates.io/crates/audioadapter):
  The core `audioadapter` traits.
- [audioadapter-sample](https://crates.io/crates/audioadapter-sample):
  A companion crate that provides sample format conversions
  as well as extensions to the standard `Read` and `Write` traits.
- [audioadapter-buffers](https://crates.io/crates/audioadapter-buffers):
  This crate, a companion crate that provides wrappers for various common data structures.

## This crate
This crate provides a selection of wrappers and buffers
that implement the `audioadapter` traits.

## Direct wrappers
The [owned] and [direct] modules
contain implementations that pass the sample values on unchanged.
These are used when the sample data is already in a usable format,
and only the data layout needs to be handled.

### Example, wrap a vector of i16 as an interleaved stereo buffer
```rust
use audioadapter::Adapter;
use audioadapter_buffers::direct::InterleavedSlice;

// Make a vector with some dummy data.
// 2 channels * 3 frames => 6 samples
let data: Vec<i16> = vec![1, 2, 3, 4, 5, 6];

// Wrap it with an interleaved adapter
let adapter = InterleavedSlice::new(&data[..], 2, 3).unwrap();

// Loop over all samples and print their values
for channel in 0..adapter.channels() {
    for frame in 0..adapter.frames() {
        let value = adapter.read_sample(channel, frame).unwrap();
        println!("Channel: {}, frame: {}, value {}", channel, frame, value);
    }
}
```


## Converting wrappers
Audio is often exchanged as buffers of raw bytes, and it is up to each application
to treat those bytes as samples of the correct format.
The [number_to_float] module is designed to help with this.

### Example, wrap a buffer of bytes containing interleaved raw samples
This shows how to read 24-bit integer format from raw bytes
while converting them to f32:

```rust
use audioadapter_buffers::number_to_float::InterleavedNumbers;
use audioadapter::Adapter;
use audioadapter_sample::sample::I24_LE;

// make a vector with some dummy data.
// 2 channels * 3 frames * 3 bytes per sample => 18 bytes
let data: Vec<u8> = vec![
    1, 1, 1, //frame 1, left
    2, 2, 2, //frame 1, right
    3, 3, 3, //frame 2, left
    4, 4, 4, //frame 2, right
    5, 5, 5, //frame 3, left
    6, 6, 6  //frame 3, right
];

// wrap the data
let buffer = InterleavedNumbers::<&[I24_LE], f32>::new_from_bytes(&data, 2, 3).unwrap();

// Loop over all samples and print their values
for channel in 0..buffer.channels() {
    for frame in 0..buffer.frames() {
        let value = buffer.read_sample(channel, frame).unwrap();
        println!(
            "Channel: {}, frame: {}, value: {}",
            channel, frame, value
        );
    }
}
```

Note that the example uses `I24_LE`, which means 24-bit samples
stored as 3 bytes in little-endian order without padding.
24-bit samples are also commonly stored with a padding byte, so that each sample takes up four bytes.
This is handled by selecting `I24_4RJ_LE` or `I24_4LJ_LE` as the format.

## Use without the standard library
This crate can be used in `no_std` environments if the `std` Cargo feature is disabled.
You can also enable the `alloc` feature to get the buffer types in the [owned] module.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
