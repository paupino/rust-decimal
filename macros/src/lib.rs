extern crate rust_decimal;
extern crate proc_macro;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use rust_decimal::Decimal;
use std::str::FromStr;

#[proc_macro]
pub fn dec(input: TokenStream) -> TokenStream {
    let mut source = input.to_string();

    // If it starts with `- ` then get rid of the extra space
    // to_string will put a space between tokens
    if source.starts_with("- ") {
        source.remove(1);
    }

    let decimal = match Decimal::from_str(&source[..]) {
        Ok(d) => d,
        Err(e) => panic!("Unexpected decimal format for {}: {}", source, e),
    };

    let unpacked = decimal.unpack();
    // We need to further unpack these for quote for now
    let lo = unpacked.lo;
    let mid = unpacked.mid;
    let hi = unpacked.hi;
    let negative = unpacked.is_negative;
    let scale = unpacked.scale;
    let expanded = quote! {
        ::rust_decimal::Decimal::from_parts(#lo, #mid, #hi, #negative, #scale)
    };
    expanded.into()
}
