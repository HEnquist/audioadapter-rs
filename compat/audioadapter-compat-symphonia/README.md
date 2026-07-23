# audioadapter-compat-symphonia

[audioadapter](https://crates.io/crates/audioadapter) trait implementations for
the planar `AudioBuffer` type from
[symphonia](https://crates.io/crates/symphonia) (via `symphonia-core`).

Wrap a `&AudioBuffer<S>` (read-only) or `&mut AudioBuffer<S>` (read-write) in a
`SymphoniaAdapter` to access it through the `audioadapter` traits. Channel
access is a plain slice, so copying is fast.

## Versioning

This crate exposes types from both [audioadapter](https://crates.io/crates/audioadapter)
and [symphonia-core](https://crates.io/crates/symphonia-core) in its public API,
so its version is tied to one major version of each.

**This `1.x` release targets audioadapter `5.x` and symphonia-core `0.6.x`.**

A new incompatible release of either dependency is supported by a new major
version of this crate. Pick the version that matches what you use:

| this crate | audioadapter | symphonia-core |
|------------|--------------|----------------|
| `1.x`      | `5.x`        | `0.6.x`        |

## License

Licensed under either of Apache-2.0 or MIT at your option.
