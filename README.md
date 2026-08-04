# Decimal &emsp; [![Build Status]][actions] [![Latest Version]][crates.io] [![Docs Badge]][docs]

[Build Status]: https://img.shields.io/endpoint.svg?url=https%3A%2F%2Factions-badge.atrox.dev%2Fpaupino%2Frust-decimal%2Fbadge&label=build&logo=none

[actions]: https://actions-badge.atrox.dev/paupino/rust-decimal/goto

[Latest Version]: https://img.shields.io/crates/v/rust-decimal.svg

[crates.io]: https://crates.io/crates/rust-decimal

[Docs Badge]: https://docs.rs/rust_decimal/badge.svg

[docs]: https://docs.rs/rust_decimal

A Decimal number implementation written in pure Rust suitable for financial calculations that require significant
integral and fractional digits with no round-off errors.

The binary representation consists of a 96 bit integer number, a scaling factor used to specify the decimal fraction and
a 1 bit sign. Because of this representation, trailing zeros are preserved and may be exposed when in string form. These
can be truncated using the `normalize` or `round_dp` functions.

# Please read before contributing

This is the **main** branch and is now considered _unstable_.

The stable 1.x version of Rust Decimal has been branched and exists in the `v1` branch. Patch requests and bug fixes are
still accepted against `v1` however will need to continue being backwards compatible and aligning with the support
model.

This branch also accepts contributions - any contributions to this branch are permitted to be breaking as we work
towards an improved Decimal API. Some things expected to break over the coming months are:

* Removal of deprecated APIs
* Removing use of panics within the code without explicit opt in
* Clean up of error types for `const` and `no-std` support
* Module system experimentation

## Installing

```sh
$ cargo add rust_decimal
```

Alternatively, you can edit your `Cargo.toml` directly and run `cargo update`:

```toml
[dependencies]
rust_decimal = "2.0.0-alpha.0"
```

To enable macro support, you can enable the `macros` feature:

```sh
$ cargo add rust_decimal --features macros
```

## Usage

Decimal numbers can be created in a few distinct ways. The easiest and most efficient method of creating a Decimal is to
use the macro:

```rust
// Import via use rust_decimal_macros or use the `macros` feature to import at the crate level
// `use rust_decimal_macros::dec;`
// or
// `use rust_decimal::dec;`

let number = dec!(-1.23) + dec!(3.45);
assert_eq!(number, dec!(2.22));
assert_eq!(number.to_string(), "2.22");
```

Alternatively you can also use one of the Decimal number convenience
functions ([see the docs](https://docs.rs/rust_decimal/) for more details):

```rust
// Using the prelude can help importing trait based functions (e.g. core::str::FromStr).
use rust_decimal::prelude::*;

// Using an integer followed by the decimal points
let scaled = Decimal::new(202, 2);
assert_eq!("2.02", scaled.to_string());

// From a 128 bit integer
let balance = Decimal::from_i128_with_scale(5_897_932_384_626_433_832, 2);
assert_eq!("58979323846264338.32", balance.to_string());

// From a string representation
let from_string = Decimal::from_str("2.02").unwrap();
assert_eq!("2.02", from_string.to_string());

// From a string representation in a different base
let from_string_base16 = Decimal::from_str_radix("ffff", 16).unwrap();
assert_eq!("65535", from_string_base16.to_string());

// From scientific notation
let sci = Decimal::from_scientific_exact("9.7e-7").unwrap();
assert_eq!("0.00000097", sci.to_string());

// Using the `Into` trait
let my_int: Decimal = 3_i32.into();
assert_eq!("3", my_int.to_string());

// Using the raw decimal representation
let pi = Decimal::from_parts(1_102_470_952, 185_874_565, 1_703_060_790, false, 28);
assert_eq!("3.1415926535897932384626433832", pi.to_string());

// If the `macros` feature is enabled, it also allows for the `dec!` macro
let amount = dec!(25.12);
assert_eq!("25.12", amount.to_string());
```

Once you have instantiated your `Decimal` number you can perform calculations with it just like any other number:

```rust
use rust_decimal::prelude::*; // Includes the `dec` macro when feature specified

let amount = dec!(25.12);
let tax_percentage = dec!(0.085);
let total = amount + (amount * tax_percentage).round_dp(2);
assert_eq!(total, dec!(27.26));
```

## Features

**Behavior / Functionality**

* [alloc](#alloc)
* [borsh](#borsh)
* [bytemuck](#bytemuck)
* [c-repr](#c-repr)
* [macros](#macros)
* [maths](#maths)
* [ndarray](#ndarray)
* [rkyv](#rkyv)
* [rust-fuzz](#rust-fuzz)
* [std](#std)
* [wasm](#wasm)

**Database**

* [db-postgres](#db-postgres)
* [db-tokio-postgres](#db-tokio-postgres)
* [db-diesel-postgres](#db-diesel-postgres)
* [db-diesel-mysql](#db-diesel-mysql)

**Serde**

* [serde-default-number](#serde-default-number)
* [serde-default-exact](#serde-default-exact)
* [Per-field helpers](#per-field-helpers)

### `align16`

Forces `Decimal`'s alignment to 16 bytes (128 bits). This is identical to `u128` and `i128`'s alignment on x86 platforms.

### `alloc`

Enables features that require heap allocation via the [`alloc`](https://doc.rust-lang.org/alloc/) crate, without
requiring `std`. Suitable for `no_std` environments that have an allocator available.

Currently this gates the `LowerExp` / `UpperExp` (scientific notation) `Display` implementations. Enabled by default.
Implied by `std`.

### `borsh`

Enables [Borsh](https://borsh.io/) serialization for `Decimal`.

### `bytemuck`

Enables [bytemuck](https://github.com/Lokathor/bytemuck) support by deriving `Pod` and `Zeroable` for `Decimal`. This also activates the `c-repr` feature since `Pod` requires `repr(C)`.

### `c-repr`

Forces `Decimal` to use `[repr(C)]`.

### `db-postgres`

Enables a PostgreSQL communication module. It allows for reading and writing the `Decimal`
type by transparently serializing/deserializing into the `NUMERIC` data type within PostgreSQL.

### `db-tokio-postgres`

Enables the tokio postgres module allowing for async communication with PostgreSQL.

### `db-diesel-postgres`

Enable [`diesel`](https://diesel.rs) PostgreSQL support.

### `db-diesel-mysql`

Enable [`diesel`](https://diesel.rs) MySQL support.

### `macros`

The `macros` feature enables a compile time macro `dec` to be available at both the crate root, and via prelude.

This parses the input at compile time and converts it to an optimized `Decimal` representation. Invalid inputs will
cause a compile time error.

Any Rust number format is supported, including scientific notation and alternate bases.

```rust
use rust_decimal::prelude::*;

assert_eq!(dec!(1.23), Decimal::new(123, 2));
```

### `maths`

The `maths` feature enables additional complex mathematical functions such as `pow`, `ln`, `enf`, `exp` etc.
Documentation detailing the additional functions can be found on the
[`MathematicalOps`](https://docs.rs/rust_decimal/latest/rust_decimal/trait.MathematicalOps.html) trait.

Please note that `ln` and `log10` will panic on invalid input with `checked_ln` and `checked_log10` the preferred
functions
to curb against this. When the `maths` feature was first developed the library would instead return `0` on invalid
input. To re-enable this
non-panicking behavior, please use the feature: `maths-nopanic`.

### `ndarray`

Enables arithmetic operations using [`ndarray`](https://github.com/rust-ndarray/ndarray) on arrays of `Decimal`.

### `proptest`

Enables a [`proptest`](https://github.com/proptest-rs/proptest) strategy to generate values for Rust Decimal.

### `rand`

Implements `rand::distributions::Distribution<Decimal>` to allow the creation of random instances.

Note: When using `rand::Rng` trait to generate a decimal between a range of two other decimals, the scale of the
randomly-generated
decimal will be the same as the scale of the input decimals (or, if the inputs have different scales, the higher of the
two).

### `rkyv`

Enables [rkyv](https://github.com/rkyv/rkyv) serialization for `Decimal`. In order to avoid breaking changes, this is
currently locked at version `0.7`.

Supports rkyv's safe API when the `rkyv-safe` feature is enabled as well.

If `rkyv` support for versions `0.8` of greater is desired, `rkyv`'
s [remote derives](https://rkyv.org/derive-macro-features/remote-derive.html) should be used instead. See
`examples/rkyv-remote`.

### `rust-fuzz`

Enable `rust-fuzz` support by implementing the `Arbitrary` trait.

### `serde-default-number`

Serializes `Decimal` as an unquoted number instead of the default quoted string.

```json
{
  "value": 1.234
}
```

On its own this converts via `f64`, so values beyond 64-bit float precision are
rounded. Combine it with `serde-default-exact` to keep full precision.

### `serde-default-exact`

Reads and writes `Decimal` with full precision rather than via `f64`, using the
`arbitrary_precision` feature of `serde_json` (added as a weak dependency).

This affects *reading* whenever it is enabled: without it, an unquoted number
carrying more precision than an `f64` can hold fails to deserialize. It affects
*writing* only when combined with `serde-default-number`.

The two features are independent, and all four combinations are supported:

| Features | `1.0000` serializes as | reads unquoted 28-digit |
| --- | --- | --- |
| *(neither)* | `"1.0000"` | error |
| `serde-default-number` | `1.0` | error |
| `serde-default-exact` | `"1.0000"` | exact |
| both | `1.0000` | exact |

Enabling `serde-default-exact` alone is a useful configuration: it writes quoted
strings, which are unambiguous, while still accepting high-precision numbers
from producers you do not control.

> **Note:** binary formats such as `bincode` and `postcard` need no configuration.
> They are detected automatically and always round-trip correctly.

### Per-field helpers

The `serde` feature always provides modules for overriding the format on
individual fields, without changing the crate-wide default:

```rust
#[derive(Serialize, Deserialize)]
pub struct Example {
    #[serde(with = "rust_decimal::serde::str")]
    as_string: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    as_float: Decimal,
    #[serde(with = "rust_decimal::serde::str_option")]
    optional: Option<Decimal>,
}
```

Available as `str`, `str_option`, `float` and `float_option`. Enabling
`serde-default-exact` additionally provides `arbitrary_precision` and
`arbitrary_precision_option`.

Each module also exposes `serialize` and `deserialize` separately, so one
direction can be overridden while the other keeps the default:

```rust
#[derive(Serialize, Deserialize)]
pub struct Example {
    #[serde(deserialize_with = "rust_decimal::serde::str::deserialize")]
    value: Decimal,
}
```

### `std`

Enables `std` library support. This is enabled by default and implies `alloc`.

This crate supports three build configurations:

| Configuration            | Cargo flags                                  |
|--------------------------|----------------------------------------------|
| `std` (default)          | (default)                                    |
| `no_std` + `alloc`       | `--no-default-features --features=alloc`     |
| `no_std` + no allocator  | `--no-default-features`                      |

The no-allocator configuration is suitable for bare-metal targets such as `x86_64-unknown-none`.

### `wasm`

Enable [`wasm-bindgen`](https://github.com/rustwasm/wasm-bindgen) support which makes `Decimal` compatible with the
`wasm_bindgen` attribute macro and exposes the following methods across boundaries:
* `fromNumber()` / `toNumber()` — convert between `Decimal` and the primitive `number` type.
* `fromString()` / `toString()` — convert between `Decimal` and a string representation.

## Building

Please refer to the [Build document](BUILD.md) for more information on building and testing Rust Decimal.

## Minimum Rust Compiler Version

The current _minimum_ compiler version is `1.67.1` which was released on `2023-02-09`.

This library maintains support for rust compiler versions that are 4 minor versions away from the current stable rust
compiler version.
For example, if the current stable compiler version is `1.50.0` then we will guarantee support up to and
including `1.46.0`.
Of note, we will only update the minimum supported version if and when required.

## Comparison to other Decimal implementations

During the development of this library, there were various design decisions made to ensure that decimal calculations
would
be quick, accurate and efficient. Some decisions, however, put limitations on what this library can do and ultimately
what
it is suitable for. One such decision was the structure of the internal decimal representation.

This library uses a mantissa of 96 bits made up of three 32-bit unsigned integers with a fourth 32-bit unsigned integer
to represent the scale/sign
(similar to the C and .NET Decimal implementations).
This structure allows us to make use of algorithmic optimizations to implement basic arithmetic; ultimately this gives
us the ability
to squeeze out performance and make it one of the fastest implementations available. The downside of this approach
however is that
the maximum number of significant digits that can be represented is roughly 28 base-10 digits (29 in some cases).

While this constraint is not an issue for many applications (e.g. when dealing with money), some applications may
require a higher number of significant digits to be represented. Fortunately,
there are alternative implementations that may be worth investigating, such as:

* [bigdecimal](https://crates.io/crates/bigdecimal)
* [decimal-rs](https://crates.io/crates/decimal-rs)

If you have further questions about the suitability of this library for your project, then feel free to either start a
[discussion](https://github.com/paupino/rust-decimal/discussions) or open
an [issue](https://github.com/paupino/rust-decimal/issues) and we'll
do our best to help.
