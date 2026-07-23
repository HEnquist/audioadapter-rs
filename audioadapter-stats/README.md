# audioadapter-stats

Statistics helpers for [audioadapter](https://crates.io/crates/audioadapter)
buffers: per-channel and per-frame RMS, minimum/maximum, and peak-to-peak.

The helpers are provided by the `AdapterStats` extension trait, which is
implemented for every type that implements `audioadapter::Adapter<T>` for a
numeric sample type. Bring the trait into scope and call the methods on any
adapter:

```rust
use audioadapter_stats::AdapterStats;
# use audioadapter::tests::MinimalAdapter;
# let data = vec![1.0_f32, 1.0, -1.0, -1.0, 1.0, 1.0, -1.0, -1.0];
# let buffer = MinimalAdapter::new_from_vec(data, 2, 4);
// `buffer` is any audioadapter `Adapter<f32>`
assert_eq!(buffer.channel_rms(0), 1.0);
assert_eq!(buffer.channel_min_and_max(0), (-1.0, 1.0));
assert_eq!(buffer.channel_peak_to_peak(0), 2.0);
```

This crate keeps the numeric dependency (`num-traits`) out of the core
`audioadapter` crate, which stays dependency-free.

## Versioning

This crate is released together with the core `audioadapter` crate and shares
its major version.

## License

Licensed under either of Apache-2.0 or MIT at your option.
