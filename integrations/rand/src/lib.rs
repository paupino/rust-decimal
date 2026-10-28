//! [`rand`](https://docs.rs/rand) distributions for [`rust_decimal::Decimal`].
//!
//! Enable `rand-0_9` and/or `rand-0_10`; each version lives in its own module. Every module
//! provides:
//!
//! * `StandardDecimal`: any `Decimal` across the full mantissa and scale range.
//! * `UniformDecimal`: a `Decimal` uniformly sampled from a range, at the inputs' scale.
#![no_std]
#![forbid(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(feature = "rand-0_10")]
pub mod rand_0_10;
#[cfg(feature = "rand-0_9")]
pub mod rand_0_9;
