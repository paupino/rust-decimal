# Decimal &emsp; [![Build Status]][travis] [![Latest Version]][crates.io]

[Build Status]: https://api.travis-ci.org/paupino/rust-decimal.svg?branch=master
[travis]: https://travis-ci.org/paupino/rust-decimal
[Latest Version]: https://img.shields.io/crates/v/rust-decimal.svg
[crates.io]: https://crates.io/crates/rust-decimal

A Decimal implementation written in pure Rust suitable for financial calculations that require significant integral and fractional digits with no round-off errors.

The binary representation consists of a 96 bit integer number, a scaling factor used to specify the decimal fraction and a 1 bit sign. Because of this representation, trailing zeros are preserved and may be exposed when in string form. These can be truncated using the `round_dp` function.

[Documentation](https://docs.rs/rust_decimal/)

## Usage

Decimal numbers can be created in a few distinct ways, depending on the rust compiler version you're targeting.

### Stable

The stable version of rust requires you to create a Decimal number using one of it's convenience methods.

```
use rust_decimal::Decimal;

// Using an integer followed by the decimal points
let scaled = Decimal::new(202, 2); // 2.02

// From a string representation
let from_string = Decimal::from_str("2.02").unwrap(); // 2.02

// Using the `Into` trait
let my_int : Decimal = 3i32.into();

// Using the raw decimal representation
// 3.1415926535897932384626433832
let pi = Decimal::from_parts(1102470952, 185874565, 1703060790, false, 28);
```

### Nightly

With the nightly version of rust you can use a procedural macro using the `rust_decimal_macros` crate. The advantage of this method is that the decimal numbers are parsed at compile time.

```
// Procedural macros need importing directly
use rust_decimal_macros::*;

let number = dec!(-1.23);
```
