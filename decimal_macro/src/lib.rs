#![feature(proc_macro)]
extern crate proc_macro;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;

#[proc_macro]
pub fn d(input: TokenStream) -> TokenStream {
    for token in input {
        panic!("test: {:?}", token);
    }
    quote!("123").parse().unwrap()
}