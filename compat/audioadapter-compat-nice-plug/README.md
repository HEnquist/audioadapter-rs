# audioadapter-compat-nice-plug

[audioadapter](https://crates.io/crates/audioadapter) trait implementations for
the `Buffer` type of the [nice-plug](https://codeberg.org/RustAudio/nice-plug)
VST3/CLAP plugin framework, via its crates.io-published core crate
[nice-plug-core](https://crates.io/crates/nice-plug-core).

`nice-plug` is a maintained continuation of
[nih-plug](https://github.com/robbert-vdh/nih-plug) and shares its planar,
non-interleaved buffer layout (one `&mut [f32]` slice per channel). Wrap a
`&Buffer` (read-only) or `&mut Buffer` (read-write) in a `NicePlugAdapter` to
feed `audioadapter`-based processing directly from a plugin's audio buffer.

## Versioning

This crate exposes types from both [audioadapter](https://crates.io/crates/audioadapter)
and [nice-plug-core](https://crates.io/crates/nice-plug-core) in its public API,
so its version is tied to one major version of each.

**This `1.x` release targets audioadapter `5.x` and nice-plug-core `0.1.x`.**

A new incompatible release of either dependency is supported by a new major
version of this crate. Pick the version that matches what you use:

| this crate | audioadapter | nice-plug-core |
|------------|--------------|----------------|
| `1.x`      | `5.x`        | `0.1.x`        |

## License

Licensed under either of Apache-2.0 or MIT at your option.
