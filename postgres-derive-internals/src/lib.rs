extern crate syn;
extern crate quote;

use syn::{MacroInput, MetaItem, Body, VariantData};
use quote::{Tokens, ToTokens};

use overrides::Overrides;
use fromsql::expand_derive_fromsql;
use tosql::expand_derive_tosql;

mod accepts;
mod composites;
mod enums;
mod fromsql;
mod overrides;
mod tosql;

pub fn expand_derive(source: &str) -> Result<String, String> {
    let mut input = try!(syn::parse_macro_input(source));
    let (tosql, fromsql) = strip_derives(&mut input);

    let tosql = if tosql {
        try!(expand_derive_tosql(&input))
    } else {
        "".to_owned()
    };
    let fromsql = if fromsql {
        try!(expand_derive_fromsql(&input))
    } else {
        "".to_owned()
    };

    strip_overrides(&mut input);

    let mut tokens = Tokens::new();
    input.to_tokens(&mut tokens);

    Ok(format!("{}{}{}", tokens, tosql, fromsql))
}

fn strip_derives(input: &mut MacroInput) -> (bool, bool) {
    let mut tosql = false;
    let mut fromsql = false;

    let mut other_attrs = vec![];
    for mut attr in input.attrs.drain(..) {
        {
            let mut items = match attr.value {
                MetaItem::List(ref name, ref mut items) if name == "derive" => items,
                _ => {
                    other_attrs.push(attr);
                    continue;
                }
            };

            items.retain(|i| {
                match *i {
                    MetaItem::Word(ref name) if name == "ToSql" => {
                        tosql = true;
                        false
                    }
                    MetaItem::Word(ref name) if name == "FromSql" => {
                        fromsql = true;
                        false
                    }
                    _ => true,
                }
            });

            if items.is_empty() {
                continue;
            }
        }

        other_attrs.push(attr);
    }

    input.attrs = other_attrs;
    (tosql, fromsql)
}

fn strip_overrides(input: &mut MacroInput) {
    Overrides::strip(&mut input.attrs);

    match input.body {
        Body::Enum(ref mut variants) => {
            for variant in variants {
                Overrides::strip(&mut variant.attrs);
            }
        }
        Body::Struct(VariantData::Struct(ref mut fields)) |
        Body::Struct(VariantData::Tuple(ref mut fields)) => {
            for field in fields {
                Overrides::strip(&mut field.attrs);
            }
        }
        Body::Struct(VariantData::Unit) => {}
    }
}
