# audiobuffer

## AudioBuffer

A simple library for making it easier to work with buffers of audio data.

Audio data can be stored in many different ways,
where both the layout of the data, and the numerical representation can vary.
This crate aims at helping with the differences in layout.

### Background
Libraries and appications that process audio usually use
a single layout for the audio data internally.
If a project combines libraries that store their audio data differently,
any data passed between them must be converted
by copying the data from a buffer using one layout
to another buffer using the other layout.

### Interleaved and sequential

explain! Sequential, planar or non-interleaved.

### Abstracting the data layout
This crate provedes a trait [AudioBuffer] that provides simple methods
for accessing the audio samples of a buffer.
It also provides wrappers for a number of common data structures
used for storing audio data.

By accessing the audio data via the trait methods instead
of indexing the data structure directly,
an application or library becomes independant of the data layout.

### Supporting new data structures
The required trait methods are simple, to make is easy to implement them for
data structures not covered by the built-in wrappers.

There are default implementations for the functions that read and write slices.
These loop over the elements to read or write and clone element by element.
These may be overriden if the wrapped data structure provides a more efficient way
of cloning the data, such as `clone_from_slice()`.

### TODO
RMS and peak calculation for numerical types 


### License: MIT

