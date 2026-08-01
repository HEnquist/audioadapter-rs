# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

Day to day:

- `cargo test --workspace --all-targets` runs everything.
- `cargo test -p audioadapter-sample --lib -- convert_I8` runs a single test or a name prefix.
  Add `--nocapture` to see printed output.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` and `cargo fmt --all`.
- `cargo bench --workspace` runs the criterion benches (`audioadapter/benches/sqrt.rs`,
  `audioadapter-buffers/benches/iteration.rs`).

The full list of checks that CI runs, including the `--no-default-features` and MSRV variants, is in
the Development section of [README.md](README.md). Run all of them before opening a PR. Note that
doc tests matter here: the crate READMEs are included as crate-level docs with
`#![doc = include_str!("../README.md")]`, so a broken example in a README fails
`cargo test --doc`.

## Workspace layout

Five kinds of crate, with a strict dependency direction:

- `audioadapter` is the core. It is `#![no_std]` and has **no dependencies** in a normal build.
  Keep it that way. `num-traits` is pulled in only by the optional `test-utils` feature, which
  exposes the generic trait-conformance helpers (`audioadapter::tests::test_float_adapter_mut_methods`
  and friends) that the other crates use in their own tests.
- `audioadapter-sample` holds sample format conversion. Depends on `num-traits` only.
- `audioadapter-buffers` holds buffer wrappers. Depends on the two above.
- `compat/audioadapter-compat-*` implement the core traits for buffer types from foreign crates
  (`audio`, `symphonia`, `dasp`, `ndarray`, `nice-plug`). Each depends only on `audioadapter` plus
  its foreign crate, never on the other companion crates.

`audioadapter-sample` and `audioadapter-buffers` are `no_std` compatible via
`#![cfg_attr(not(feature = "std"), no_std)]` with `std` on by default. `audioadapter-buffers` also
has an `alloc` feature, which gates the `owned` module. Anything new must respect these gates.

## Architecture

**The core abstraction.** `Adapter<T>` and `AdapterMut<T>` give indexed `read_sample(channel, frame)`
/ `write_sample` access, hiding both the memory layout and the stored sample type. Everything else in
the workspace is either an implementation of these traits or a wrapper that implements them in terms
of another adapter. `stats` (`AdapterStats`, `StatsSample`) and `iterators` (`AdapterIterators`) are
extension traits blanket-implemented for any adapter.

Both traits are `unsafe trait`. The contract is not about the `_unchecked` methods but about
`channels()` and `frames()`: they must report correct and stable bounds, because the safe helper
methods bounds-check against them and then call the unchecked accessors.

**Two conversion layers.** In `audioadapter-sample`:

- `RawSample` converts a numeric type to and from a float scaled to -1.0..+1.0.
- `BytesSample` converts raw bytes to and from the nearest numeric type, and is implemented for the
  byte-wrapper newtypes (`I16_LE`, `I24_4LJ_BE`, `F32_LE`, ...), each a `[u8; N]` newtype.

A blanket `impl<V: BytesSample> RawSample for V` chains them, so a byte wrapper gets float conversion
for free as soon as its `BytesSample` impl exists. The 24-bit formats are hand-written because they
have no matching primitive and are widened to `i32`/`u32`; everything else goes through the
`bytessample_for_newtype!` macro.

Scaling conventions worth knowing: signed integers map `MIN` to -1.0, so the range is asymmetric and
+1.0 is not reachable. Unsigned integers are offset binary, centred at half the range (128 for `u8`).
Out of range floats clip to the integer limits and set `ConversionResult::clipped` rather than
erroring. Float formats are never clipped, since values outside -1.0..+1.0 are valid headroom.

**Wrapper families in `audioadapter-buffers`.** Three modules that are easy to confuse:

- `direct` wraps a slice whose elements are already the wanted type (`InterleavedSlice`,
  `SequentialSliceOfVecs`, the `Sparse*` variants, ...). No conversion.
- `number_to_float` wraps a *slice* of numeric or byte-wrapper samples and reads it as floats
  (`InterleavedNumbers`, `SequentialNumbers`).
- `adapter_to_float` wraps an *existing adapter* and converts on the fly (`ConvertNumbers` over a
  numeric adapter, `ConvertBytes` over an adapter of `[u8; N]` arrays).

`PlainBytes` in `number_to_float` is an unsafe marker trait. The `new_from_bytes` constructors
transmute `&[u8]` into `&[SampleType]`, which is only sound for types with alignment 1 and no invalid
bit patterns. Implement it only for `[u8; N]` newtypes.

### Adding a sample format

A new byte-wrapper format needs edits in three places, and it is easy to miss the last two:

1. `audioadapter-sample/src/sample.rs`: the newtype plus a `bytessample_for_newtype!` line.
2. `audioadapter-buffers/src/number_to_float.rs`: the import list and `impl_plainbytes!`.
3. `audioadapter-buffers/src/adapter_to_float.rs`: a `byte_convert_traits_newtype!` line.

## Conventions

- Tests live in `#[cfg(test)] mod tests` at the bottom of each module. Per-format tests are generated
  by declarative macros (`test_simple_int_bytes!`, `test_to_signed_int!`, ...); prefer extending an
  existing macro invocation list over writing a new one-off test, but write explicit tests where a
  format has behaviour the macro does not capture.
- The crates are versioned and released independently. Bump only the crates that actually changed.
  The version numbers happen to line up for the older releases because those changes touched every
  crate, so do not read that history as a lockstep policy. Record user visible changes in the
  workspace-level [CHANGELOG.md](CHANGELOG.md) under `Unreleased`, listing the crates that changed
  and their next versions.
- The core `audioadapter` crate is the stable point of the family and is meant to stay on its
  current major version for a long time. Prefer solutions that leave its public API alone, and treat
  a change that would force a major bump there as a decision to raise rather than make.
- When one workspace crate starts using new API from another, raise the dependency requirement to
  that version in the manifest, not just the version of the crate itself. `audioadapter-buffers`
  requires `audioadapter-sample` 5.1.0 for exactly this reason: it uses `I8` and `U8`, and a
  requirement of 5.0.0 would let a resolver pick a version that does not compile.
- Edition 2024, MSRV 1.87 declared per crate as `rust-version`. The CI MSRV job reads the lowest
  `rust-version` in the workspace, so bumping it means bumping every manifest.

### Releasing

- Before making a release, check whether any of the crates the compat crates target have had new
  releases. The `release_checks` job in [ci-test.yml](.github/workflows/ci-test.yml) does this on
  every push, so read its warnings on the latest run. Locally, `cargo update --dry-run --verbose`
  reports the same thing as `Unchanged <crate> vA (available: vB)` lines: those are the updates
  cargo cannot take on its own, either because they are semver-incompatible or because they need a
  newer Rust than the MSRV. Retargeting a new version belongs in the same release as the version
  bump, since it may change the MSRV and the compat crate version.
- The `## Unreleased` heading for the release must be replaced with the release date as
  `## YYYY-MM-DD`, matching the older entries. Released changes must never be left sitting under
  `Unreleased`. Pushing a tag triggers the publish workflow and publishes immediately, so date the
  changelog before tagging. If a release turns out to already be published and its entry still says
  `Unreleased`, fix the heading using the date of the release tag.
- The same `release_checks` job warns whenever the newest changelog heading is not a `YYYY-MM-DD`
  date, which also catches a heading that is misspelled or carries a version number instead of a
  date. That warning is expected during normal development, when the newest heading is `Unreleased`.
  It is there so the reminder is in front of you in the pull request that prepares a release.
- Both checks are warning-level and never fail the build, matching the publish dry-run job. An
  upstream release or a not-yet-dated changelog should not block an unrelated pull request.
