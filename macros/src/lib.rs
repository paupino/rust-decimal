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

mod str;

use proc_macro::TokenStream;
use quote::quote;

/// Transform a literal number directly to a `Decimal` at compile time. Any Rust number format works.
///
/// - `dec!(1)`, `dec!(-1)`, `dec!(1_999)`, `dec!(- 1_999)`
/// - `dec!(0b1)`, `dec!(-0b1_1111)`, `dec!(0o1)`, `dec!(-0o1_777)`, `dec!(0x1)`, `dec!(-0x1_Ffff)`
/// - `dec!(1.)`, `dec!(-1.111_009)`, `dec!(1e6)`, `dec!(-1.2e+6)`, `dec!(12e-6)`, `dec!(-1.2e-6)`
///
/// ### Option `radix:`
///
/// You can give it integers (not float-like) in any radix from 2 to 36 inclusive, using the letters too:
/// `dec!(radix: 2, 100) == 4`, `dec!(radix: 3, -1_222) == -53`, `dec!(radix: 36, z1) == 1261`,
/// `dec!(radix: 36, -1_xyz) == -90683`
///
/// ### Option `exp:`
///
/// This is the same as the `e` 10’s exponent in float syntax (except as a Rust expression it doesn’t accept
/// a unary `+`.) You need this for other radices. Currently, it must be between -28 and +28 inclusive:
/// `dec!(radix: 2, exp: 5, 10) == 200_000`, `dec!(exp: -3, radix: 8, -1_777) == dec!(-1.023)`
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
/// let number = dec!(-0o1_777);
/// assert_eq!("-1023", number.to_string());
/// ```
///
#[proc_macro]
pub fn dec(input: TokenStream) -> TokenStream {
    let input_str = input.to_string();
    
    // Parse input to extract radix, exp, and value
    let mut radix: Option<u32> = None;
    let mut exp: Option<i32> = None;
    let mut value = String::new();
    
    // Check if we have named arguments
    if input_str.contains("radix:") || input_str.contains("exp:") {
        // Split by commas 
        let parts: Vec<&str> = input_str.split(',').collect();
        
        // Find the parts that contain our named parameters
        for part in parts.iter() {
            let trimmed = part.trim();
            
            if trimmed.starts_with("radix:") {
                let radix_str = trimmed.trim_start_matches("radix:").trim();
                radix = Some(radix_str.parse().unwrap_or_else(|_| {
                    panic!("Invalid radix value: {}", radix_str)
                }));
            } else if trimmed.starts_with("exp:") {
                let exp_str = trimmed.trim_start_matches("exp:").trim();
                exp = Some(exp_str.parse().unwrap_or_else(|_| {
                    panic!("Invalid exp value: {}", exp_str)
                }));
            } else {
                // The last non-named part is the value
                value = trimmed.to_string();
            }
        }
    } else {
        // Just a regular value with no named arguments
        value = input_str;
    }
    
    // Process the parsed input
    let result = if let Some(radix) = radix {
        str::parse_radix_dec(radix, &value, exp.unwrap_or_default())
    } else {
        str::parse_dec(&value, exp.unwrap_or_default())
    };
    
    let unpacked = match result {
        Ok(d) => d,
        Err(e) => panic!("{}", e),
    };
    
    expand(
        unpacked.lo(),
        unpacked.mid(),
        unpacked.hi(),
        unpacked.negative(),
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
