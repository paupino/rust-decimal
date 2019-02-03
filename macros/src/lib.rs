extern crate proc_macro_hack;
extern crate rust_decimal_macro_impls;

use proc_macro_hack::proc_macro_hack;

#[proc_macro_hack]
pub use rust_decimal_macro_impls::dec;
