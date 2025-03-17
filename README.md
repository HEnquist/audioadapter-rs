# audioadapter

The `audioadapter` library simplifies working with audio data buffers.

Audio data can vary in layout and numerical representation.
This crate bridges these differences, handling both layout and data types effectively.

It does not introduce new data structures for storing audio data.
Instead, it acts as an adapter, leveraging existing structures
like vectors and arrays to store raw audio data.
The library provides adapter wrappers for these structures.
The adapters implement traits that define a common set of methods
for accessing and modifying audio samples.
These methods enable the development of applications that can read
and write audio data regardless of its layout and numerical representation.

## Background
Libraries and applications that process audio usually use
a single layout for the audio data internally.
If a project combines libraries that store their audio data differently,
any data passed between them must be converted
by copying the data from a buffer using one layout
to another buffer using the other layout.

## Channels and frames
When audio data has more than one channel is made up of a series of _frames_.
A frame consists of the samples for all channels, belonging to one time point.
For normal stereo, a frame consists of one sample for the left channel
and one for the right, usually stored in that order.

## Interleaved and sequential
When audio data is stored in a file or in memory,
the data can be ordered in two main ways.
- Keeping all samples for each channel together,
  and storing each channel after the previous.
  This is normally called _sequential_, _non-interleaved_ or _planar_.
  The sample order of a stereo file with 3 frames becomes:
  `L1, L2, L3, R1, R2, R3`
- Keeping all samples for each frame together,
  and storing each frame after the previous.
  This is normally called _interleaved_, and this is how the data in a .wav file is ordered.
  The sample order of a stereo file with 3 frames becomes:
  `L1, R1, L2, R2, L3, R3`

In a more general sense, the same applies when storing
any multi-dimensional array in linear storage such as RAM or a file.
A 2D matrix can then be stored in _row-major_ or _column-major_ order.
The only difference here compared to a general 2D matrix is that the names `row` and `column`
are replaced by the audio-specific `channel` and `frame`.
Using the general notation, _interleaved_ corresponds to _frame-major_ order,
and _sequential_ to _channel-major_ order.

### Choosing the best order
A project that uses `audioadapter` supports both sequential and interleaved buffers,
but depending on how the data is processed, one order may give better performance than the other.

To get the best performance, use the layout that stores the samples in memory
in the same order as they are accessed during processing.
This makes memory accesses very predicable, which helps the CPU cache to maximize memory throughput.
If there is no obvious most common processing order,
try both and measure the performance.

#### Interleaved
Use this if the project processes the data frame by frame, such as this dummy loop:
```ignore
for frame in 0..data.frames() {
    for channel in 0..data.channels() {
        do_something(&data, channel, frame);
    }
}
```

#### Sequential
Use this if the project processes the data channel by channel:
```ignore
for channel in 0..data.channels() {
    for frame in 0..data.frames() {
        do_something(&data, channel, frame);
    }
}
```

## Abstracting the data layout
This module provides the traits [Adapter] and [AdapterMut].
These enable basic reading and writing, with methods that access the sample values
indirectly.
This makes it possible to do implementations where the samples are converted
from one format to another when reading and writing from/to the underlying data.

The crate also provides wrappers that implement the traits some or all of these traits
for a number of common data structures used for storing audio data.

Any type implementing [std::clone::Clone] can be used as the type for the samples.
This includes for example all the usual numeric types (`u8`, `f32` etc),
as well as arrays and vectors of numbers (`Vec<i32>`, `[u8; 4]` etc). 

By accessing the audio data via the trait methods instead
of indexing the data structure directly,
an application or library becomes independant of how the data is stored.

## Handling buffers of raw bytes
Audio is often exchanged as buffers of raw bytes, and it is up to each application
to treat those bytes as samples of the correct format.
The [number_to_float] module is desgined to help with this.

Example, wrap a buffer of bytes containing interleaved raw samples in 24-bit integer format,
while converting them to f32:
```rust
use audioadapter::number_to_float::InterleavedNumbers;
use audioadapter::Adapter;
use audioadapter::sample::I24LE;

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
let buffer = InterleavedNumbers::<&[I24LE<3>], f32>::new_from_bytes(&data, 2, 3).unwrap();

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

Note that the example uses `I24LE<3>`, which means 24-bit samples
stored as 3 bytes in little-endian order without padding.
24-bit samples are also commonly stored with a padding byte, so that each sample takes up four bytes.
This is handled by selecting `I24LE<4>` as the format.

## Reading and writing samples from types implementing `Read` and `Write`
The [std::io::Read] and [std::io::Write] traits are useful for reading
and writing raw bytes to and from for example files.
The [readwrite] module adds methods for reading and writing samples,
with on-the-fly conversion between bytes and the numerical values.

Example
```rust
use audioadapter::sample::I16LE;
use audioadapter::readwrite::ReadSamples;

// make a vector with some dummy data.
let data: Vec<u8> = vec![1, 2, 3, 4];
// slices implement Read.
let mut slice = &data[..];
// read the first value as 16 bit integer, convert to f32.
let float_value = slice.read_converted::<I16LE, f32>();
```

## Compatibility with the [audio](https://crates.io/crates/audio) crate
In addition to the provided wrappers, the [Adapter], [AdapterMut] traits are implemented for
buffers implementing the [audio_core::Buf], [audio_core::BufMut] and [audio_core::ExactSizeBuf]
traits from the [audio](https://crates.io/crates/audio) crate.
This is enabled via the `audio` Cargo feature, which is enabled by default.

Example: Create a buffer and access it using [Adapter] methods.
```
use audioadapter::Adapter;
use audio;

let buf: audio::buf::Interleaved<i32> = audio::buf::Interleaved::with_topology(2, 4);
# #[cfg(feature = "audio")]
buf.read_sample(0,0);
```


## Supporting new data structures
The required trait methods are simple, to make is easy to implement them for
data structures not covered by the built-in wrappers.

There are default implementations for the functions that read and write slices.
These loop over the elements to read or write and clone element by element.
These may be overriden if the wrapped data structure provides a more efficient way
of cloning the data, such as [slice::clone_from_slice()].

See also the `custom_adapter` example.
This shows an implementation of [Adapter]
for a vector of strings.

### Ideas for future improvements
- index iterator (providing channel, frame)
- supporting reuse of wrapping adapters by replacing inner data

## License: MIT
