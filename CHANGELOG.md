# Changelog

Notable changes to the crates in this workspace. Each release is dated and lists
the crates that were published, with only the changed crates shown.

## Unreleased

`audioadapter` 5.0.0, `audioadapter-sample` 5.0.0, `audioadapter-buffers` 5.0.0,
`audioadapter-compat-*` 1.0.0 (new)

- Move buffer-crate integrations to separate `audioadapter-compat-*` crates
- `audio` support moved out of core, no longer a default feature
- Core `audioadapter` now has no dependencies
- `stats` no longer uses `num-traits`, sample types now need the new `StatsSample` trait
- Add `channel_peak`, `frame_peak`, `channel_mean`, `frame_mean` and the raw `_sum` and
  `_sum_of_squares` methods to `AdapterStats`
- Fix min/max and peak-to-peak for signals that never cross zero, such as unsigned samples
- Drop unused `audio` feature from `audioadapter-sample`
- Target `nice-plug-core` 0.2 in `audioadapter-compat-nice-plug`
- Bump MSRV to 1.87, required by `nice-plug-core` 0.2

## 2026-06-17

`audioadapter` 4.0.0, `audioadapter-sample` 4.0.0, `audioadapter-buffers` 4.0.0

- Add `copy_from_other` and a slice-copy `utils` module
- Remove vestigial `'a` lifetime from `Adapter` and `AdapterMut`
- Fix byte-slice soundness (alignment asserts, `PlainBytes`, `BytesSample::zero`)
- Reject integer overflow in bounds checks

## 2026-03-31

`audioadapter` 3.0.0, `audioadapter-sample` 3.0.0, `audioadapter-buffers` 3.0.0

- Mark `Adapter` and `AdapterMut` as `unsafe` traits
- Improved error types
- Remove `sqrt` from the public API
- Drop the `libm` dependency
- Add `SequentialSliceOfSlices`
- Bump MSRV and edition, switch to dual MIT/Apache-2.0 license

## 2026-03-24

`audioadapter` 2.0.1

- Fix `no_std` support
- Fix frame RMS calculation

## 2025-12-08

`audioadapter` 2.0.0, `audioadapter-sample` 2.0.0, `audioadapter-buffers` 2.0.0

- New naming convention for sample formats
- Support left-justified 24-bit formats
- Add more basic trait methods

## 2025-10-24

`audioadapter` 1.0.0, `audioadapter-sample` 1.0.0, `audioadapter-buffers` 1.0.0

- First stable release
- Split into `audioadapter`, `audioadapter-sample` and `audioadapter-buffers`
- More intuitive method names
