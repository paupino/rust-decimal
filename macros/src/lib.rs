//!
//! A helpful macro for instantiating `Decimal` numbers.
//!
//! By default, this requires `rust_decimal` to be available at the project root. e.g. the macro
//! will effectively produce:
//!
//! ```ignore
//! ::rust_decimal::Decimal::from_parts(12345, 0, 0, false, 4)
//! ```
//!
//! While this is convenient for most use cases, it is sometimes not desired behavior when looking
//! to reexport the libray. Consequently, this behavior can be modified by enabling the feature
//! `reexportable`. When this feature is enabled, the macro will instead reproduce the functional
//! equivalent of:
//!
//! ```ignore
//! Decimal::from_parts(12345, 0, 0, false, 4)
//! ```
//!
//! # Examples
//!
//! ```rust
//! use rust_decimal_macros::dec;
//!
//! // If the reexportable feature is enabled, `Decimal` needs to be in scope
//! #[cfg(feature = "reexportable")]
//! use rust_decimal::Decimal;
//!
//! let number = dec!(1.2345);
//! assert_eq!("1.2345", number.to_string());
//! let number = dec!(-5.4321);
//! assert_eq!("-5.4321", number.to_string());
//! ```
//!

use core::str::FromStr;
use proc_macro::TokenStream;
use quote::quote;
use rust_decimal::Decimal;

#[proc_macro]
pub fn dec(input: TokenStream) -> TokenStream {
    let mut source = input.to_string();

    // If it starts with `- ` then get rid of the extra space
    // to_string will put a space between tokens
    if source.starts_with("- ") {
        source.remove(1);
    }

    let decimal = match Decimal::from_str(&source[..]).or_else(|_| Decimal::from_scientific(&source[..])) {
        Ok(d) => d,
        Err(e) => panic!("Unexpected decimal format for {}: {}", source, e),
    };

    let unpacked = decimal.unpack();
    expand(
        unpacked.lo,
        unpacked.mid,
        unpacked.hi,
        unpacked.negative,
        unpacked.scale,
    )
}

#[cfg(not(feature = "reexportable"))]
fn expand(lo: u32, mid: u32, hi: u32, negative: bool, scale: u32) -> TokenStream {
    let expanded = quote! {
        ::rust_decimal::Decimal::from_parts(#lo, #mid, #hi, #negative, #scale)
    };
    expanded.into()
}

#[cfg(feature = "reexportable")]
fn expand(lo: u32, mid: u32, hi: u32, negative: bool, scale: u32) -> TokenStream {
    let expanded = quote! {
        Decimal::from_parts(#lo, #mid, #hi, #negative, #scale)
    };
    expanded.into()
}
