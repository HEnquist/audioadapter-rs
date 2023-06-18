# audiobuffer

## AudioBuffer

A simple library for making it easier to work with buffers of audio data.

Audio data can be stored in many different ways,
where both the layout of the data, and the numerical representation can vary.
This crate aims at helping with the differences in layout.

#### Background
Libraries and applications that process audio usually use
a single layout for the audio data internally.
If a project combines libraries that store their audio data differently,
any data passed between them must be converted
by copying the data from a buffer using one layout
to another buffer using the other layout.

#### Channels and frames
When audio data has more than one channel is made up of a series of _frames_.
A frame consists of the samples for all channels, belonging to one time point.
For normal stereo, a frame consists of one sample for the left channel
and one for the right, usually in that order.

#### Interleaved and sequential
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

#### Abstracting the data layout
This crate provedes a trait [AudioBuffer] that provides simple methods
for accessing the audio samples of a buffer.
It also provides wrappers for a number of common data structures
used for storing audio data.
Any type implementing [std::clone::Clone] can be used as the type for the samples.

By accessing the audio data via the trait methods instead
of indexing the data structure directly,
an application or library becomes independant of the data layout.

#### Supporting new data structures
The required trait methods are simple, to make is easy to implement them for
data structures not covered by the built-in wrappers.

There are default implementations for the functions that read and write slices.
These loop over the elements to read or write and clone element by element.
These may be overriden if the wrapped data structure provides a more efficient way
of cloning the data, such as [slice::clone_from_slice()].

#### RMS and peak calculation

The `AudioBufferStats` trait provides methods for calculating the RMS and peak-to-peak values
for channels and frames.
This is only available when the samples are of a numerical kind, such as integers or floats,
and cannot be used when the samples are for example arrays of bytes such as `[u8; 4]`.


#### License: MIT


- MaybeInterleaved
- AbstractAudio
- AnyOrder
InterleaveItOrNot
IWantToInterleave

