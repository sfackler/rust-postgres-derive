#![recursion_limit = "256"]

extern crate syn;
#[macro_use]
extern crate quote;

mod accepts;
mod composites;
mod enums;
mod fromsql;
mod overrides;
mod tosql;

pub fn expand_derive_tosql(source: &str) -> Result<String, String> {
    let input = try!(syn::parse_macro_input(source));
    tosql::expand_derive_tosql(&input)
}

pub fn expand_derive_fromsql(source: &str) -> Result<String, String> {
    let input = try!(syn::parse_macro_input(source));
    fromsql::expand_derive_fromsql(&input)
}
