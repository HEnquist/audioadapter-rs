# audioadapter-compat-dasp

[audioadapter](https://crates.io/crates/audioadapter) trait implementations for
slices of [dasp](https://crates.io/crates/dasp) frames (`&[F]` / `&mut [F]`
where `F: dasp_frame::Frame`).

A `dasp` multi-channel buffer is a sequence of frames, each holding one sample
per channel (for example `[f32; 2]` for stereo). Wrap the slice in a
`DaspAdapter` to access it through the `audioadapter` traits.

## Versioning

This crate exposes types from both [audioadapter](https://crates.io/crates/audioadapter)
and [dasp_frame](https://crates.io/crates/dasp_frame) in its public API, so its
version is tied to one major version of each.

**This `1.x` release targets audioadapter `5.x` and dasp / dasp_frame `0.11.x`.**

A new incompatible release of either dependency is supported by a new major
version of this crate. Pick the version that matches what you use:

| this crate | audioadapter | dasp_frame |
|------------|--------------|------------|
| `1.x`      | `5.x`        | `0.11.x`   |

## Changelog

See the [changelog](https://github.com/HEnquist/audioadapter-rs/blob/master/CHANGELOG.md).

## License

Licensed under either of Apache-2.0 or MIT at your option.
