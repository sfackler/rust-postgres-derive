#![feature(rustc_macro, rustc_macro_lib)]

extern crate rustc_macro;
extern crate postgres_derive_internals;

use rustc_macro::TokenStream;

#[rustc_macro_derive(ToSql)]
pub fn derive_tosql(input: TokenStream) -> TokenStream {
    match postgres_derive_internals::expand_derive_tosql(&input.to_string()) {
        Ok(expanded) => expanded.parse().unwrap(),
        Err(err) => panic!(err),
    }
}
