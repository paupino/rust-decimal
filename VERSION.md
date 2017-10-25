# Version History

## 0.5.1

* Minor bux fix update. Prevent `panic` upon parsing an empty string.

## 0.5.0

* Removes postgres from default feature set.
* `bincode` support for serde
* Better support for format strings
* Benchmarks added to tests

## 0.4.2

Fixes bug in `cmp` whereby negative's were not being compared correctly.

## 0.4.1

Minor bugfix to support creating negative numbers using the default constructor.

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
