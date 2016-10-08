#![feature(proc_macro, proc_macro_lib)]

extern crate proc_macro;
extern crate postgres_derive_internals;

use proc_macro::TokenStream;

#[proc_macro_derive(ToSql)]
pub fn derive_tosql(input: TokenStream) -> TokenStream {
    derive("ToSql", input)
}

#[proc_macro_derive(FromSql)]
pub fn derive_fromsql(input: TokenStream) -> TokenStream {
    derive("FromSql", input)
}

fn derive(trait_: &str, input: TokenStream) -> TokenStream {
    let input = format!("#[derive({})] {}", trait_, input);
    match postgres_derive_internals::expand_derive(&input) {
        Ok(expanded) => expanded.parse().unwrap(),
        Err(err) => panic!(err),
    }
}
