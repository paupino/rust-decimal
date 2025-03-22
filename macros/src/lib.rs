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
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, Expr, Ident, LitInt, Result, Token,
};

/// Transform a literal number directly to a `Decimal` at compile time.
///
/// Any Rust number format works, for example:
///
/// - `dec!(1)`, `dec!(-1)`, `dec!(1_999)`, `dec!(- 1_999)`
/// - `dec!(0b1)`, `dec!(-0b1_1111)`, `dec!(0o1)`, `dec!(-0o1_777)`, `dec!(0x1)`, `dec!(-0x1_Ffff)`
/// - `dec!(1.)`, `dec!(-1.111_009)`, `dec!(1e6)`, `dec!(-1.2e+6)`, `dec!(12e-6)`, `dec!(-1.2e-6)`
///
/// ### Option `radix`
///
/// You can give it integers (not float-like) in any radix from 2 to 36 inclusive, using the letters too:
/// `dec!(100, radix 2) == 4`, `dec!(-1_222, radix 3) == -53`, `dec!(z1, radix 36) == 1261`,
/// `dec!(-1_xyz, radix 36) == -90683`
///
/// ### Option `exp`
///
/// This is the same as the `e` 10's exponent in float syntax (except as a Rust expression it doesn't accept
/// a unary `+`.) You need this for other radixes. Currently, it must be between -28 and +28 inclusive:
/// `dec!(10, radix 2, exp 5) == 200_000`, `dec!( -1_777, exp -3, radix 8) == dec!(-1.023)`
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
/// let number = dec!(-1_777, radix 8);
/// assert_eq!("-1023", number.to_string());
/// ```
///
#[proc_macro]
pub fn dec(input: TokenStream) -> TokenStream {
    // Parse the input using our custom parser
    let dec_input = parse_macro_input!(input as DecInputParser);
    let value = dec_input.value.as_str();

    // Process the parsed input
    let result = if let Some(radix) = dec_input.radix {
        str::parse_decimal_with_radix(value, dec_input.exp.unwrap_or_default(), radix)
    } else {
        str::parse_decimal(value, dec_input.exp.unwrap_or_default())
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

/// Custom parser for the dec! macro input
struct DecInputParser {
    radix: Option<u32>,
    exp: Option<i32>,
    value: String,
}

impl Parse for DecInputParser {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut radix = None;
        let mut exp = None;
        let mut value = None;

        // Parse the first item which could be a value or a parameter
        if !input.is_empty() {
            // Try to parse as an identifier (parameter name)
            if input.peek(Ident) {
                let ident = input.parse::<Ident>()?;
                let ident_str = ident.to_string();

                match ident_str.as_str() {
                    "radix" => {
                        if let Some(value) = parse_radix(input)? {
                            radix = Some(value);
                        }
                    }
                    "exp" => {
                        if let Some(value) = parse_exp(input)? {
                            exp = Some(value);
                        }
                    }
                    _ => {
                        // This is not a parameter but a value
                        value = Some(ident_str);
                    }
                }
            } else {
                // It's a value
                let expr = input.parse::<Expr>()?;
                value = Some(quote!(#expr).to_string());
            }
        }

        // Parse the remaining tokens
        while !input.is_empty() {
            // We expect a comma between tokens
            if input.peek(Token![,]) {
                let _ = input.parse::<Token![,]>()?;
            }

            // Check if we're at the end
            if input.is_empty() {
                break;
            }

            // Parse the next token
            if input.peek(Ident) {
                let ident = input.parse::<Ident>()?;
                let ident_str = ident.to_string();

                match ident_str.as_str() {
                    "radix" => {
                        if radix.is_some() {
                            panic!("Duplicate radix parameter");
                        }
                        if let Some(value) = parse_radix(input)? {
                            radix = Some(value);
                        }
                    }
                    "exp" => {
                        if exp.is_some() {
                            panic!("Duplicate exp parameter");
                        }
                        if let Some(value) = parse_exp(input)? {
                            exp = Some(value);
                        }
                    }
                    _ => {
                        // This is not a parameter but a value
                        if value.is_none() {
                            value = Some(ident_str);
                        } else {
                            panic!("Unknown parameter or duplicate value: {}", ident_str);
                        }
                    }
                }
            } else {
                // Parse as an expression (value)
                if value.is_none() {
                    let expr = input.parse::<Expr>()?;
                    value = Some(quote!(#expr).to_string());
                } else {
                    panic!("Duplicate value found");
                }
            }
        }

        // Ensure we have a value
        let value = value.unwrap_or_else(|| panic!("Expected a decimal value"));

        Ok(DecInputParser { radix, exp, value })
    }
}

fn parse_radix(input: ParseStream) -> Result<Option<u32>> {
    // Parse the value after the parameter name
    if input.peek(LitInt) {
        let lit_int = input.parse::<LitInt>()?;
        return Ok(Some(lit_int.base10_parse::<u32>()?));
    }
    let expr = input.parse::<Expr>()?;
    match expr {
        Expr::Lit(lit) => {
            if let syn::Lit::Int(lit_int) = lit.lit {
                return Ok(Some(lit_int.base10_parse::<u32>()?));
            }
        }
        _ => panic!("Expected a literal integer for radix"),
    }

    Ok(None)
}

fn parse_exp(input: ParseStream) -> Result<Option<i32>> {
    // Parse the value after the parameter name
    if input.peek(LitInt) {
        let lit_int = input.parse::<LitInt>()?;
        return Ok(Some(lit_int.base10_parse::<i32>()?));
    }
    let expr = input.parse::<Expr>()?;
    match expr {
        Expr::Lit(lit) => {
            if let syn::Lit::Int(lit_int) = lit.lit {
                return Ok(Some(lit_int.base10_parse::<i32>()?));
            }
        }
        Expr::Unary(unary) => {
            if let Expr::Lit(lit) = *unary.expr {
                if let syn::Lit::Int(lit_int) = lit.lit {
                    let mut val = lit_int.base10_parse::<i32>()?;
                    if let syn::UnOp::Neg(_) = unary.op {
                        val = -val;
                    }
                    return Ok(Some(val));
                }
            }
        }
        _ => panic!("Expected a literal integer for exp"),
    }
    Ok(None)
}
