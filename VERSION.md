# Version History

## 0.6.2

Fixes an issue with division of rational numbers allowing results greater than `MAX_PRECISION`. This would ultimately cause issues for future operations on this number.
In addition, in some cases transitive operations would not be equal due to overflow being lost.

## 0.6.1

This minor release is purely to expose `rust_decimal_macros` for use on the nightly channel. Documentation has been updated accordingly.

## 0.6.0

This release has a few major changes to the internal workings of the `Decimal` implementation and consequently comes with a number of performance improvements.

* Floats can now be parsed into a `Decimal` type using `from_f32` and `from_f64`.
* `add`, `sub`, `mul` run roughly 1500% faster than before.
* `div` run's roughly 1000% faster than before with room for future improvement.
* Also get significant speed improvements with `cmp`, `rescale`, `round_dp` and some string manipulations.
* Implemented `*Assign` traits for simpler usage.
* Removed `BigInt` and `BigUint` as being intermediary data types.

## 0.5.2

Minor bug fix to prevent a `panic` from overflow during comparison of high significant digit decimals. 

## 0.5.1

Minor bux fix to prevent `panic` upon parsing an empty string.

## 0.5.0

* Removes postgres from default feature set.
* `bincode` support for serde
* Better support for format strings
* Benchmarks added to tests

## 0.4.2

Fixes bug in `cmp` whereby negative's were not being compared correctly.

## 0.4.1

Minor bug fix to support creating negative numbers using the default constructor.

## 0.4.0

This release is a stylistic cleanup however does include some minor changes that may break existing builds.

### Changed
* Serde is now optional. You can enable Serde support within `features` using the keyword `serde`.
* Serde now returns errors on invalid input as opposed to `0`.
* `f64` conversion support has been added.
* Update Postgres dependency to use v0.15.

## 0.3.1

This is a documentation release that should help with discoverability and usage.

## 0.3.0

### Changed
* Removed trait `ToDecimal` and replaced with builtin [`From`](https://doc.rust-lang.org/std/convert/trait.From.html) trait ([`#12`](https://github.com/paupino/rust-decimal/pull/12))
