# audioadapter-compat-audio

[audioadapter](https://crates.io/crates/audioadapter) trait implementations for
buffers from the [audio](https://crates.io/crates/audio) crate.

Wrap an `audio` buffer in an `AudioBufAdapter` to access it through the
`audioadapter` `Adapter` / `AdapterMut` traits.

```rust
use audioadapter::Adapter;
use audioadapter_compat_audio::AudioBufAdapter;

let buf = audio::wrap::interleaved(&[1, 2, 3, 4, 5, 6, 7, 8], 2);
let adapter = AudioBufAdapter::new(buf);
assert_eq!(adapter.read_sample(0, 0), Some(1));
```

## Versioning

This crate exposes types from both [audioadapter](https://crates.io/crates/audioadapter)
and [audio](https://crates.io/crates/audio) in its public API, so its version is
tied to one major version of each.

**This `1.x` release targets audioadapter `5.x` and audio `0.2.x`.**

A new incompatible release of either dependency is supported by a new major
version of this crate. Pick the version that matches what you use:

| this crate | audioadapter | audio |
|------------|--------------|-------|
| `1.x`      | `5.x`        | `0.2.x` |

## Changelog

See the [changelog](https://github.com/HEnquist/audioadapter-rs/blob/master/CHANGELOG.md).

## License

Licensed under either of Apache-2.0 or MIT at your option.
