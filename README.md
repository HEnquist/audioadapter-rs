# audioadapter

A library for making it easier to work with buffers of audio data.

Audio data can be stored in many different ways,
where both the layout of the data, and the numerical representation can vary.
The `audioadapter` crate aims at helping with the differences
in layout both layout and data type.

This crate does not provide any data structures of its own
for storing the audio data.
Instead it functions as an adapter.
The "raw" audio data should be stored in existing structures
such as the built-in vectors and arrays.
The crate then provides adapters for these.
An adapter consist of a wrapping structure that provides
a set of common methods for reading and writing the sample values
from the wrapped data structure. 


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
and one for the right, usually in that order.

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
This module provides several "layers" of traits that add more functionality.
The most basic traits are [Indirect] and [IndirectMut].
These enable basic reading and writing, with methods that access the sample values
indirectly.

The next level is the [Direct] and [DirectMut] traits,
adding methods that access the samples directly.
This includes immutable and immutable borrowing, as well as iterators.

The last level is [Numeric] that is used to calculate some properties of the audio data.
This is implemented for every structure implementing [Direct],
and is only available when the samples are of a numerical kind, such as integers or floats.
It cannot be used when the samples are for example arrays of bytes such as `[u8; 4]`.

The crate also provides wrappers that implement these some or all of these traits
for a number of common data structures used for storing audio data.
Any type implementing [std::clone::Clone] can be used as the type for the samples.

By accessing the audio data via the trait methods instead
of indexing the data structure directly,
an application or library becomes independant of the data layout.

## Compatibility with the [audio] crate
In addition to the provided wrappers, the [Indirect], [IndirectMut],
[Direct] (TODO) and [DirectMut] (TODO) traits are implemented for
buffers implementing the [audio_core::Buf], [audio_core::BufMut] and [audio_core::ExactSizeBuf]
traits from the [audio] crate.
This is enabled via the `audio` Cargo feature, which is enabled by default.

Example: Create a buffer and access it using [Indirect] methods.
```
use audioadapter::Indirect;
use audio;

let buf: audio::buf::Interleaved<i32> = audio::buf::Interleaved::with_topology(2, 4);
buf.read(0,0);
```


## Supporting new data structures
The required trait methods are simple, to make is easy to implement them for
data structures not covered by the built-in wrappers.

There are default implementations for the functions that read and write slices.
These loop over the elements to read or write and clone element by element.
These may be overriden if the wrapped data structure provides a more efficient way
of cloning the data, such as [slice::clone_from_slice()].

See also the `custom_adapter` example.
This shows an implementation of [Indirect]
for a vector of strings.

## TODO
- Add method for clearing a buffer
- Implement `Direct` for `audio` crate buffers

### Possible future improvements
- separate length and capacity
- methods for resizing (within data capacity)
- methods for selective clearing (frame 0..n, n..end etc)
- wrappers for `&[AsRef<T>]`?

## License: MIT
