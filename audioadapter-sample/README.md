# audioadapter-sample

A crate that helps with handling audio samples in various formats.

This crate is part of the `audioadapter` family.

The `audioadapter` family consists of three crates:
- [audioadapter](https://crates.io/crates/audioadapter):
  The core `audioadapter` traits.
- [audioadapter-sample](https://crates.io/crates/audioadapter-sample):
  A companion crate that provides sample format conversions
  as well as extensions to the standard `Read` and `Write` traits.
- [audioadapter-buffers](https://crates.io/crates/audioadapter-buffers):
  This crate, a companion crate that provides wrappers for various common data structures.

## Introduction
Audio data is stored and exchanged in various numerical formats,
for example 16 or 24 bit integers, or 32-bit floating point.
When data is exchanged via files or APIs,
the sample values have to be stored in some binary format.
This crate is intended to help with both conversions between the various numerical formats,
as well as handling reading and writing these samples as raw bytes.

## Sample formats
This crate supports signed and unsigned integer samples with 8, 16, 24, 32 and 64 bits.
Floating point samples are also supported, with 32 and 64 bits.
For all formats, both little-endian and big-endian representations are supported.

When converting between integers and floating point, the value range of the integer
is mapped to the floating point range of -1.0 to +1.0.
If a floating point value outside this range is converted to integer,
the value will be clamped at the minimum or maximum value of the integer.

## This crate
The main functionality of this crate is provided by two main traits:
- [sample::BytesSample] - conversions between raw bytes and the (nearest) corresponding numeric type.
- [sample::RawSample] - conversions between different numeric types.

In addition to the traits, it also defines a number of wrappers for bytes.
For example, four bytes might be a signed or unsigned 32-bit integer, a 32-bit float, or
a 24-bit integer with a padding byte.
By using the appropriate wrapper, for example [sample::I32_LE], it becomes clear that
these bytes store a signed 32-bit integer in little-endian byte order.

## Special note on 24-bit audio formats

While 16-bit (`i16`) and 32-bit (`i32`) audio samples align with standard integer types,
24-bit samples do not have a direct native equivalent.

To preserve full precision when working with 24-bit data,
this crate uses the next larger integer type, `i32` or `u32`, for conversions.


### Storage formats for 24-bit samples

24-bit audio samples are typically stored in one of two formats:

- **Packed**: Each sample uses exactly 3 bytes.
- **Padded**: An additional byte is added, making each sample 4 bytes.

#### Packed format

Packed samples are space-efficient, using only the necessary 3 bytes per sample.
Since all bytes contain data, there's no ambiguity about byte placement.
However, the 3-byte alignment can be inconvenient to work with,
and certain hardware may not support it.

#### Padded format

Padded samples align to 4 bytes, which simplifies memory access
and may be required by some devices or APIs.
The padding byte can be placed in two ways:

- **Left-justified**: Padding is added as the least significant byte,
  so the actual data occupies the three most significant bytes.
  - Used in `.wav` files and the Windows Wasapi API.

- **Right-justified**: Padding is added as the most significant byte,
  placing the data in the three least significant bytes.
  - Used in the ALSA API on Linux, such as the `SND_PCM_FORMAT_S24_LE` format.

### Example: read 24-bit integers from raw bytes
```rust
use audioadapter_sample::sample::I24_LE;
use audioadapter_sample::sample::BytesSample;

// Make a vector with some dummy data.
// 9 bytes corresponding to 3 samples.
let bytes = vec![1, 2, 3, 4, 5, 6, 7, 8, 9];

for sample in bytes.chunks_exact(3) {
  let new_value: i32 = I24_LE::from_slice(sample).to_number();
  println!("{}", new_value);
}
```

## Converting between numerical formats
A very common sample format is 16-bit signed integer.
But when doing any kind of processing of audio data, such as filtering or mixing,
it is often desirable to work with floating point values.
An application may therefore need to convert input data from 16-bit integers
to floating point, perform the needed processing,
and convert back to 16-bit integers before returning the data.

### Example, converting [i16] to and from [f32]
```rust
use audioadapter_sample::sample::RawSample;

// make a vector with some dummy data.
let indata: Vec<i16> = vec![1, 2, 3, 4];
let mut outdata: Vec<i16> = Vec::new();

for sample in indata.iter() {
  // convert to f32
  let float_sample: f32 = sample.to_scaled_float();
  // do some processing
  let processed_sample = 0.99 * float_sample;
  // convert back to 16-bit integer
  let processed_int = i16::from_scaled_float(processed_sample);
  if processed_int.clipped {
    println!("The value was clipped during conversion to integer");
  }
  outdata.push(processed_int.value);
}
```

### Example, converting 16-bit integers from raw bytes to [f32]
```rust
use audioadapter_sample::sample::I16_LE;
use audioadapter_sample::sample::BytesSample;
use audioadapter_sample::sample::RawSample;

// Make a vector with some dummy data.
// 6 bytes corresponding to 3 samples.
let bytes = vec![1, 2, 3, 4, 5, 6];

for sample in bytes.chunks_exact(2) {
  let new_value: f32 = I16_LE::from_slice(sample).to_scaled_float();
  println!("{}", new_value);
}
```


## Reading and writing samples from types implementing `Read` and `Write`
The [std::io::Read] and [std::io::Write] traits are useful for reading
and writing raw bytes to and from for example files.
The [readwrite] module extends these traits by providing methods for reading and writing samples,
with on-the-fly conversion between bytes and the numerical values.
This functionality depends on the standard library and is gated by the `std` Cargo feature.

Example
```rust
# #[cfg(feature = "std")]
# {
use audioadapter_sample::sample::I16_LE;
use audioadapter_sample::readwrite::ReadSamples;

// make a vector with some dummy data.
let data: Vec<u8> = vec![1, 2, 3, 4];
// slices implement Read.
let mut slice = &data[..];
// read the first value as 16 bit integer, convert to f32.
let float_value = slice.read_converted::<I16_LE, f32>();
# }
```

## Compatibility with the [audio](https://crates.io/crates/audio) crate
The wrappers for bytes implement the [audio_core::Sample] trait from the [audio](https://crates.io/crates/audio) crate.
This is controlled by the `audio` feature which is enabled by default.

## Cargo features
This crate has the following features:
 - `std` - enables the standard library (Enabled by default)
 - `audio` - enables `audio` crate compatibility (Enabled by default)

## Use without the standard library
This crate can be used in `no_std` environments if the `std` Cargo feature is disabled.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
