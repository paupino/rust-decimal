# Version History

## 0.8.0

* Introduces `from_scientific` allowing parsing of scientific notation into the Decimal type.
* Fixes a bug when formatting a number with a leading zero's.

## 0.7.2

* Fixes bug in `rescale` whereby scaling which invoked rounding incorrectly set the new scale for the left/right sides.

## 0.7.1

* Fixes bug in `cmp` whereby two negatives would return an incorrect result.
* Further documentation examples
* Small improvements in division logic
* New `abs`, `floor` and `ceil` functions.

## 0.7.0

This is a minor version bump as we slowly build our way towards 1.0. Thank you for everyone's support and help as we get there! This has a few notable changes - also introducing a few new interfaces which is the reason for the version bump:

* `from_parts` function to allow effective creation of `Decimal`'s without requiring binary serialization. An example of this benefit is with the lazy static group initializers for Postgres.
* `normalize` function to allow stripping trailing zero's easily.
* `trunc` function allows truncation of a number without any rounding. This effectively "truncates" the fractional part of the number.
* `fract` function returns the fractional part of the number without the integral.
* Minor improvements in some iterator logic, utilizing the compiler for further optimizations.
* Fixes issue in string parsing logic whereby `_` would cause numbers to be incorrectly identified.
* Many improvements to `mul`. Numbers utilizing the `lo` portion of the decimal only will now be shortcut and bigger numbers will now correctly overflow. True overflows will still panic, however large underflows will now be rounded as necessary as opposed to panicing.
* `Hash` was implemented by convention in `0.6.5` however is reimplemented explicitly in `0.7.0` for effectiveness.
* PostgreSQL read performance improved by pre-caching groups and leveraging `normalize` (i.e. avoiding strings). Further optimizations can be made in write however require some `div` optimizations first.
* Added short circuit write improvement for zero in PostgreSQL writes.
* Benchmarks are now recorded per build so we can start tracking where slow downs have occurred. This does mean there is a performance hit on Travis builds however hopefully the pay off will make it worthwhile.

## 0.6.5

Fixes issue with rescale sometimes causing a silent overflow which led to incorrect results during addition, subtraction and compare. Consequently Decimal now rounds the most significant number so that these operations work successfully.

In addition, Decimal now derive's the `Hash` trait so that it can be used for indexing.

## 0.6.4

Fixes silent overflow errors when parsing highly significant strings. `from_str` will now round in these scenario's, similar to oleaut32 behavior.

## 0.6.3

Fixes a regression in ordering where by different scales would be rescaled towards losing precision instead of increasing precision. Have added numerous test suites to help cover more issues like this in the future.
Also fixes an issue in parsing invalid strings whereby the precision exceeded our maximum precision. Previously, this would work with unintended results however this now returns an Error returned from `FromStr`.

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
