use std::iter;
use syn::Ident;
use quote::Tokens;

use enums::Variant;
use composites::Field;

pub fn base_body(schema: &str, name: &str) -> Tokens {
    quote! {
        if type_.schema() != #schema || type_.name() != #name {
            return false;
        }
    }
}

pub fn enum_body(schema: &str, name: &str, variants: &[Variant]) -> Tokens {
    let num_variants = variants.len();
    let variant_names = variants.iter().map(|v| &v.name);

    let base = base_body(schema, name);

    quote! {
        #base

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

pub fn composite_body(schema: &str, name: &str, trait_: &str, fields: &[Field]) -> Tokens {
    let num_fields = fields.len();
    let trait_ = Ident::new(trait_);
    let traits = iter::repeat(&trait_);
    let field_names = fields.iter().map(|f| &f.name);
    let field_types = fields.iter().map(|f| &f.type_);

    let base = base_body(schema, name);

    quote! {
        #base

        match *type_.kind() {
            ::postgres::types::Kind::Composite(ref fields) => {
                if fields.len() != #num_fields {
                    return false;
                }

                fields.iter().all(|f| {
                    match f.name() {
                        #(
                            #field_names => {
                                <#field_types as ::postgres::types::#traits>::accepts(f.type_())
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
