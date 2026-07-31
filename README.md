# audioadapter

The `audioadapter` family of crates simplifies working with audio data buffers.

Audio data can vary in layout and numerical representation.
This crate bridges these differences, handling both layout and data types effectively.

The `audioadapter` family has three core crates:
- [audioadapter](https://crates.io/crates/audioadapter):
  that provides the traits for reading and writing audio data,
  and helpers for calculating statistics such as RMS and peak values.
- [audioadapter-sample](https://crates.io/crates/audioadapter-sample): A companion crate
  that provides sample format conversions as well as extensions to the standard `Read` and `Write` traits.
- [audioadapter-buffers](https://crates.io/crates/audioadapter-buffers): A companion crate
  that provides wrappers for various common data structures.

In addition to these, the `audioadapter-compat-*` crates implement the traits for
buffer types from other audio crates. See the
[core crate documentation](https://docs.rs/audioadapter) for the current list.


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

## Development

Before opening a PR, run the same checks as CI.

Required checks (must pass):

- `cargo check --workspace --all-targets` - Type-checks all crates and targets quickly without running tests.
- `cargo test --workspace --all-targets` - Runs unit, integration, and doc tests across the workspace.
- `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps` - Builds docs and fails on rustdoc warnings.
- `cargo bench --workspace --no-run` - Compiles benchmarks to catch benchmark-only build errors.
- `cargo check --workspace --no-default-features` - Verifies all crates compile with default features disabled.
- `cargo check --workspace --all-targets --no-default-features` - Extends the no-default-features check to tests/examples/benches.
- `cargo test --workspace --no-default-features` - Runs tests with default features disabled.
- `cargo fmt --all -- --check` - Verifies formatting matches `rustfmt` output.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` - Runs Clippy and treats all lints as errors.
- `cargo +1.87 check --workspace --all-targets --all-features` - Verifies that everything builds with the
  oldest supported Rust version, the `rust-version` declared in the manifests.

Publish dry-runs (warning-level checks):

- `cargo publish --dry-run -p audioadapter` - Validates that the base crate can be packaged and published.
- `cargo publish --dry-run -p audioadapter-sample` - Validates packaging/publishability of the sample companion crate.
- `cargo publish --dry-run -p audioadapter-buffers` - Validates packaging/publishability of the buffers companion crate.
- `cargo publish --dry-run -p audioadapter-compat-<name>` - Validates packaging/publishability of each compat crate.

Dry-run publish checks are a useful signal, but they may fail in valid situations
(for example, due to crates.io dependency/version timing). Treat failures here as
"review required" rather than as an automatic blocker.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
