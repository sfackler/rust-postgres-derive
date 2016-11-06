#![feature(proc_macro, proc_macro_lib)]

extern crate proc_macro;
extern crate postgres_derive_internals;
#[macro_use]
extern crate post_expansion;
extern crate syn;
extern crate quote;

use proc_macro::TokenStream;
use quote::{ToTokens, Tokens};

register_post_expansion!(PostExpansion_postgres_derive);

#[proc_macro_derive(ToSql)]
pub fn derive_tosql(input: TokenStream) -> TokenStream {
    derive(input, postgres_derive_internals::expand_derive_tosql)
}

#[proc_macro_derive(FromSql)]
pub fn derive_fromsql(input: TokenStream) -> TokenStream {
    derive(input, postgres_derive_internals::expand_derive_fromsql)
}

fn derive(input: TokenStream, expand: fn(&str) -> Result<String, String>) -> TokenStream {
    let source = input.to_string();

    let decl = expand_decl(&source);

    let impl_ = match expand(&source) {
        Ok(impl_) => impl_,
        Err(e) => panic!("{}", e),
    };

    let expanded = format!("{}\n{}", decl, impl_);
    expanded.parse().unwrap()
}

fn expand_decl(source: &str) -> String {
    let ast = syn::parse_macro_input(source).unwrap();
    let stripped = post_expansion::strip_attrs_later(ast, &["postgres"], "postgres_derive");

    let mut tokens = Tokens::new();
    stripped.to_tokens(&mut tokens);
    tokens.to_string()
}
