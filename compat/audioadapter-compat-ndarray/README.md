# audioadapter-compat-ndarray

[audioadapter](https://crates.io/crates/audioadapter) trait implementations for
two-dimensional [ndarray](https://crates.io/crates/ndarray) arrays
(`ArrayBase<_, Ix2>`, including `Array2<T>` and array views).

The axis order is chosen explicitly:

* `NdarrayAdapter::new_channels_frames` for arrays shaped `(channels, frames)`.
* `NdarrayAdapter::new_frames_channels` for arrays shaped `(frames, channels)`.

Sample access uses ndarray indexing, so it is correct for any memory layout, and
the bulk copy helpers take a contiguous-slice fast path in the common
standard-layout case.

## Versioning

This crate exposes types from both [audioadapter](https://crates.io/crates/audioadapter)
and [ndarray](https://crates.io/crates/ndarray) in its public API, so its version
is tied to one major version of each.

**This `1.x` release targets audioadapter `5.x` and ndarray `0.17.x`.**

A new incompatible release of either dependency is supported by a new major
version of this crate. Pick the version that matches what you use:

| this crate | audioadapter | ndarray |
|------------|--------------|---------|
| `1.x`      | `5.x`        | `0.17.x` |

## License

Licensed under either of Apache-2.0 or MIT at your option.
