# audioadapter

The `audioadapter` family of crates simplifies working with audio data buffers.

Audio data can vary in layout and numerical representation.
This crate bridges these differences, handling both layout and data types effectively.

The `audioadapter` family consists of three crates:
- [audioadapter](https://crates.io/crates/audioadapter):
  that provides the traits for reading and writing audio data.
- [audioadapter-sample](https://crates.io/crates/audioadapter-sample): A companion crate
  that provides sample format conversions as well as extensions to the standard `Read` and `Write` traits.
- [audioadapter-buffers](https://crates.io/crates/audioadapter-buffers): A companion crate
  that provides wrappers for various common data structures.


## Purpose of audioadapter
Libraries and applications that process audio usually use
a single layout for the audio data internally.
If a project combines libraries that store their audio data differently,
any data passed between them must be converted
by copying the data from a buffer using one layout
to another buffer using the other layout.

Similarly, an application may process audio data using one type,
for example 16-bit integers or 32-bit floats.
Applications that process audio often use floating point,
while audio typically is stored in integer formats with 16, 24 or 32 bits.
Applications thus need to handle the conversion from whatever format
the input data has to its internal processing format,
and then again to the desired output format.

The audioadapter crates help with both these challenges.

## Safety and `unsafe trait`

The core traits `Adapter` and `AdapterMut` are `unsafe trait`s.
Here, `unsafe` does not mean that *using* the trait methods is always unsafe,
but that implementing the traits has a safety contract.

The contract is that methods like `channels()` and `frames()` must always report
correct, stable bounds for the underlying buffer while the adapter is in use.
Many safe helper methods rely on these values before calling unchecked access
internally. If an implementation reports incorrect values, those internal unchecked
reads/writes can go out of bounds and cause undefined behavior.

In short: the danger is not in the method names themselves, but in providing an
incorrect implementation of the safe size-reporting methods that the traits rely on.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
