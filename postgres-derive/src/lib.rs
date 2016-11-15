#![feature(proc_macro, proc_macro_lib)]

extern crate proc_macro;
extern crate postgres_derive_internals;

extern crate syn;

use proc_macro::TokenStream;

#[proc_macro_derive(ToSql, attributes(postgres))]
pub fn derive_tosql(input: TokenStream) -> TokenStream {
    derive(input, postgres_derive_internals::expand_derive_tosql)
}

#[proc_macro_derive(FromSql, attributes(postgres))]
pub fn derive_fromsql(input: TokenStream) -> TokenStream {
    derive(input, postgres_derive_internals::expand_derive_fromsql)
}

fn derive(input: TokenStream, expand: fn(&str) -> Result<String, String>) -> TokenStream {
    match expand(&input.to_string()) {
        Ok(impl_) => impl_.parse().unwrap(),
        Err(e) => panic!("{}", e),
    }
}
