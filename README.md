# Decimal &emsp; [![Build Status]][travis] [![Latest Version]][crates.io]

[![Join the chat at https://gitter.im/rust-decimal/Lobby](https://badges.gitter.im/rust-decimal/Lobby.svg)](https://gitter.im/rust-decimal/Lobby?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge&utm_content=badge)

[Build Status]: https://api.travis-ci.org/paupino/rust-decimal.svg?branch=master
[travis]: https://travis-ci.org/paupino/rust-decimal
[Latest Version]: https://img.shields.io/crates/v/rust-decimal.svg
[crates.io]: https://crates.io/crates/rust-decimal

A Decimal implementation written in pure Rust suitable for financial calculations that require significant integral and fractional digits with no round-off errors.

The binary representation consists of a 96 bit integer number, a scaling factor used to specify the decimal fraction and a 1 bit sign. Because of this representation, trailing zeros are preserved and may be exposed when in string form. These can be truncated using the `round_dp` function.

## Usage
Currently, creating the decimal requires either specifying the scale upon creation, using a standard primitive type or parsing a string.

```
let scaled = Decimal::new(202, 2); // 2.02
let from_string = Decimal::from_str("2.02").unwrap(); // 2.02
let my_int : Decimal = 3i32.into();
```

Future versions will investigate the use of compiler extensions and macros to simplify this process.
