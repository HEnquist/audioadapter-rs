# audioadapter

The `audioadapter` library simplifies working with audio data buffers.

Audio data can vary in layout and numerical representation.
This crate bridges these differences, handling both layout and data types effectively.

The `audioadapter` family consists of three crates:
- [audioadapter](https://crates.io/crates/audioadapter):
  This crate provides the traits such as [Adapter] and [AdapterMut].
- [audioadapter-sample](https://crates.io/crates/audioadapter-sample): A companion crate
  that provides sample format conversions as well as extensions to the standard `Read` and `Write` traits.
- [audioadapter-buffers](https://crates.io/crates/audioadapter-buffers): A companion crate
  that provides wrappers for various common data structures.


## Background
Libraries and applications that process audio usually use
a single layout for the audio data internally.
If a project combines libraries that store their audio data differently,
any data passed between them must be converted
by copying the data from a buffer using one layout
to another buffer using the other layout.

## Channels and frames
When audio data has more than one channel, it is made up of a series of _frames_.
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
  This is normally called _interleaved_, and this is how the data in most
  audio file formats such as .wav is ordered.
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
This makes memory accesses very predictable, which helps the CPU cache to maximize memory throughput.
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

## Abstracting the data layout and sample format
This module provides the traits [Adapter] and [AdapterMut].
These enable basic reading and writing, with methods that access the sample values
indirectly.
This makes it possible to do implementations where the samples are converted
from one format to another when reading and writing from/to the underlying data.

## Safety and `unsafe trait`

The core traits [Adapter] and [AdapterMut] are `unsafe trait`s.
This means that implementing the traits comes with a safety contract.

Implementations must report correct, stable bounds via `channels()` and `frames()`
while an adapter is in use.
Safe helper methods rely on these values before calling unchecked access internally.
If these methods report incorrect bounds, the internal unchecked reads/writes may go
out of bounds and cause undefined behavior.

The crate also provides wrappers that implement some or all of these traits
for a number of common data structures used for storing audio data.

Any type implementing [core::clone::Clone] can be used as the type for the samples.
This includes for example all the usual numeric types (`u8`, `f32` etc),
as well as arrays and vectors of numbers (`Vec<i32>`, `[u8; 4]` etc). 

By accessing the audio data via the trait methods instead
of indexing the data structure directly,
an application or library becomes independent of how the data is stored.

The companion crate [audioadapter-buffers](https://crates.io/crates/audioadapter-buffers)
contains wrappers that implement the `audioadapter` traits
that allow reading and writing samples from buffers of raw bytes,
with optional conversion to and from floating point values.
The format conversions are performed using the
[audioadapter-sample](https://crates.io/crates/audioadapter-sample) crate.

## Compatibility with the [audio](https://crates.io/crates/audio) crate
The [Adapter] and [AdapterMut] traits are implemented for
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
The required trait methods are simple, in order to make it easy
to implement them for new data structures.
The tests of this crate includes a minimal implementation called `MinimalAdapter`,
that is based on a normal vector. 

Since [Adapter] and [AdapterMut] are unsafe traits, new implementations must be
declared with `unsafe impl` and uphold the bounds-reporting contract above.

The [Adapter] and [AdapterMut] traits provide default implementations
for the functions that read and write slices.
These loop over the elements to read or write and clone element by element.
These may be overridden if the wrapped data structure provides a more efficient way
of cloning the data, such as [slice::clone_from_slice()].

See also the `custom_adapter` example.
This shows a minimal implementation of [Adapter]
for a vector of strings.

## Using without the standard library
The `audioadapter` traits do not require the standard library,
and can therefore be used in `no_std` environments.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
