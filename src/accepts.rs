use std::iter;
use syn::{Ident, Lifetime};
use quote::Tokens;

use crate::enums::Variant;
use crate::composites::Field;

pub fn enum_body(name: &str, variants: &[Variant]) -> Tokens {
    let num_variants = variants.len();
    let variant_names = variants.iter().map(|v| &v.name);

    quote! {
        if type_.name() != #name {
            return false;
        }

        match *type_.kind() {
            ::postgres::types::Kind::Enum(ref variants) => {
                if variants.len() != #num_variants {
                    return false;
                }

                variants.iter().all(|v| {
                    match &**v {
                        #(
                            #variant_names => true,
                        )*
                        _ => false,
                    }
                })
            }
            _ => false,
        }
    }
}

pub fn composite_body(name: &str, trait_: &str, lifetime: Option<Lifetime>, fields: &[Field]) -> Tokens {
    let num_fields = fields.len();
    let trait_ = Ident::from(trait_);
    let field_names = fields.iter().map(|f| &f.name);
    let field_types = fields.iter().map(|f| &f.type_);

    let trait_tokens = iter::repeat(match lifetime {
        Some(lifetime) => quote!{ ::postgres::types::#trait_<#lifetime> },
        None => quote! { ::postgres::types::#trait_ },
    });

    quote! {
        if type_.name() != #name {
            return false;
        }

        match *type_.kind() {
            ::postgres::types::Kind::Composite(ref fields) => {
                if fields.len() != #num_fields {
                    return false;
                }

                fields.iter().all(|f| {
                    match f.name() {
                        #(
                            #field_names => {
                                <#field_types as #trait_tokens>::accepts(f.type_())
                            }
                        )*
                        _ => false,
                    }
                })
            }
            _ => false,
        }
    }
}
