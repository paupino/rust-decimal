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
//! to reexport the library. Consequently, this behavior can be modified by enabling the feature
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

use proc_macro::TokenStream;
use quote::quote;

/// Convenience function for creating decimal numbers
///
/// # Example
///
/// ```rust
/// use rust_decimal_macros::dec;
///
/// // If the reexportable feature is enabled, `Decimal` needs to be in scope
/// #[cfg(feature = "reexportable")]
/// use rust_decimal::Decimal;
///
/// let number = dec!(1.2345);
/// assert_eq!("1.2345", number.to_string());
/// let number = dec!(-5.4321);
/// assert_eq!("-5.4321", number.to_string());
/// ```
#[proc_macro]
pub fn dec(input: TokenStream) -> TokenStream {
    let mut source = input.to_string();

    // If it starts with `- ` then get rid of the extra space
    // to_string will put a space between tokens
    if source.starts_with("- ") {
        source.remove(1);
    }

    let decimal = if source.contains('e') || source.contains('E') {
        match decimal_parser::parse_scientific(&source[..]) {
            Ok(d) => d,
            Err(e) => panic!("{}", e),
        }
    } else {
        match decimal_parser::parse_radix_10_exact(&source[..]) {
            Ok(d) => d,
            Err(e) => panic!("{}", e),
        }
    };
    if decimal.mid == 0 && decimal.hi == 0 {
        match (decimal.lo, decimal.scale, decimal.negative) {
            (0, 0, _) => return constant(Constant::Zero),
            (1, 0, false) => return constant(Constant::One),
            (1, 0, true) => return constant(Constant::NegativeOne),
            (2, 0, false) => return constant(Constant::Two),
            (10, 0, false) => return constant(Constant::Ten),
            (100, 0, false) => return constant(Constant::OneHundred),
            (1000, 0, false) => return constant(Constant::OneThousand),
            _ => {}
        }
    }

    expand(decimal.lo, decimal.mid, decimal.hi, decimal.negative, decimal.scale)
}

#[derive(Debug)]
enum Constant {
    Zero,
    One,
    Two,
    Ten,
    OneHundred,
    OneThousand,
    NegativeOne,
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

#[cfg(not(feature = "reexportable"))]
fn constant(constant: Constant) -> TokenStream {
    match constant {
        Constant::Zero => quote! { ::rust_decimal::Decimal::ZERO },
        Constant::One => quote! { ::rust_decimal::Decimal::ONE },
        Constant::Two => quote! { ::rust_decimal::Decimal::TWO },
        Constant::Ten => quote! { ::rust_decimal::Decimal::TEN },
        Constant::OneHundred => quote! { ::rust_decimal::Decimal::ONE_HUNDRED },
        Constant::OneThousand => quote! { ::rust_decimal::Decimal::ONE_THOUSAND },
        Constant::NegativeOne => quote! { ::rust_decimal::Decimal::NEGATIVE_ONE },
    }
    .into()
}

#[cfg(feature = "reexportable")]
fn constant(constant: Constant) -> TokenStream {
    match constant {
        Constant::Zero => quote! { Decimal::ZERO },
        Constant::One => quote! { Decimal::ONE },
        Constant::Two => quote! { Decimal::TWO },
        Constant::Ten => quote! { Decimal::TEN },
        Constant::OneHundred => quote! { Decimal::ONE_HUNDRED },
        Constant::OneThousand => quote! { Decimal::ONE_THOUSAND },
        Constant::NegativeOne => quote! { Decimal::NEGATIVE_ONE },
    }
    .into()
}
