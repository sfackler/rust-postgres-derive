use proc_macro2::Span;
use std::iter;
use syn::{self, Ident, DeriveInput, Data, DataStruct, Fields, Lifetime};
use quote::Tokens;

use crate::accepts;
use crate::composites::Field;
use crate::enums::Variant;
use crate::overrides::Overrides;

pub fn expand_derive_fromsql(input: DeriveInput) -> Result<Tokens, String> {
    let overrides = Overrides::extract(&input.attrs)?;

    let name = overrides.name.unwrap_or_else(|| input.ident.to_string());

    let trait_lifetime = Lifetime::new("'from_sql", Span::call_site());

    let (accepts_body, to_sql_body) = match input.data {
        Data::Enum(ref data) => {
            let variants = data.variants.iter().map(Variant::parse).collect::<Result<Vec<_>, _>>()?;
            (accepts::enum_body(&name, &variants), enum_body(&input.ident, &variants))
        }
        Data::Struct(DataStruct { fields: Fields::Unnamed(ref fields), .. }) if fields.unnamed.len() == 1 => {
            let field = fields.unnamed.first().unwrap().into_value();
            (domain_accepts_body(&name, field), domain_body(&input.ident, field))
        }
        Data::Struct(DataStruct { fields: Fields::Named(ref fields), .. }) => {
            let fields = fields.named.iter().map(Field::parse).collect::<Result<Vec<_>, _>>()?;
            (accepts::composite_body(&name, "FromSql", Some(trait_lifetime), &fields),
             composite_body(&input.ident, trait_lifetime, &fields))
        }
        _ => {
            return Err("#[derive(FromSql)] may only be applied to structs, single field tuple \
                        structs, and enums".to_owned())
        }
    };

    let ident = &input.ident;
    let out = quote! {
        impl<#trait_lifetime> ::postgres::types::FromSql<#trait_lifetime> for #ident {
            fn from_sql(_type: &::postgres::types::Type,
                        buf: &[u8])
                        -> ::std::result::Result<#ident,
                                                 ::std::boxed::Box<::std::error::Error +
                                                                   ::std::marker::Sync +
                                                                   ::std::marker::Send>> {
                #to_sql_body
            }

            fn accepts(type_: &::postgres::types::Type) -> bool {
                #accepts_body
            }
        }
    };

    Ok(out)
}

fn enum_body(ident: &Ident, variants: &[Variant]) -> Tokens {
    let variant_names = variants.iter().map(|v| &v.name);
    let idents = iter::repeat(ident);
    let variant_idents = variants.iter().map(|v| &v.ident);

    quote! {
        match ::std::str::from_utf8(buf)? {
            #(
                #variant_names => ::std::result::Result::Ok(#idents::#variant_idents),
            )*
            s => {
                ::std::result::Result::Err(
                    ::std::convert::Into::into(format!("invalid variant `{}`", s)))
            }
        }
    }
}

// Domains are sometimes but not always just represented by the bare type (!?)
fn domain_accepts_body(name: &str, field: &syn::Field) -> Tokens {
    let ty = &field.ty;
    let normal_body = accepts::domain_body(name, field);

    quote! {
        if <#ty as ::postgres::types::FromSql>::accepts(type_) {
            return true;
        }

        #normal_body
    }
}

fn domain_body(ident: &Ident, field: &syn::Field) -> Tokens {
    let ty = &field.ty;
    quote! {
        <#ty as ::postgres::types::FromSql>::from_sql(_type, buf).map(#ident)
    }
}

fn composite_body(ident: &Ident, lifetime: syn::Lifetime, fields: &[Field]) -> Tokens {
    let temp_vars = &fields.iter().map(|f| Ident::from(format!("__{}", f.ident))).collect::<Vec<_>>();
    let field_names = &fields.iter().map(|f| &f.name).collect::<Vec<_>>();
    let field_idents = &fields.iter().map(|f| &f.ident).collect::<Vec<_>>();

    quote! {
        fn read_be_i32(buf: &mut &[u8]) -> ::std::io::Result<i32> {
            let mut bytes = [0; 4];
            ::std::io::Read::read_exact(buf, &mut bytes)?;
            let num = ((bytes[0] as i32) << 24) |
                ((bytes[1] as i32) << 16) |
                ((bytes[2] as i32) << 8) |
                (bytes[3] as i32);
            ::std::result::Result::Ok(num)
        }

        fn read_value<#lifetime, T>(type_: &::postgres::types::Type,
                         buf: &#lifetime mut &[u8])
                         -> ::std::result::Result<T,
                             ::std::boxed::Box<::std::error::Error +
                             ::std::marker::Sync +
                             ::std::marker::Send>>
                         where T: ::postgres::types::FromSql<#lifetime>
        {
            let len = read_be_i32(buf)?;
            let value = if len < 0 {
                ::std::option::Option::None
            } else {
                if len as usize > buf.len() {
                    return ::std::result::Result::Err(
                        ::std::convert::Into::into("invalid buffer size"));
                }
                let (head, tail) = buf.split_at(len as usize);
                *buf = tail;
                ::std::option::Option::Some(&head[..])
            };
            ::postgres::types::FromSql::from_sql_nullable(type_, value)
        }

        let fields = match *_type.kind() {
            ::postgres::types::Kind::Composite(ref fields) => fields,
            _ => unreachable!(),
        };

        let mut buf = buf;
        let num_fields = read_be_i32(&mut buf)?;
        if num_fields as usize != fields.len() {
            return ::std::result::Result::Err(
                ::std::convert::Into::into(format!("invalid field count: {} vs {}", num_fields,
                                                   fields.len())));
        }

        #(
            let mut #temp_vars = ::std::option::Option::None;
        )*

        for field in fields {
            let oid = read_be_i32(&mut buf)? as u32;
            if oid != field.type_().oid() {
                return ::std::result::Result::Err(::std::convert::Into::into("unexpected OID"));
            }

            match field.name() {
                #(
                    #field_names => {
                        #temp_vars = ::std::option::Option::Some(
                            read_value(field.type_(), &mut buf)?);
                    }
                )*
                _ => unreachable!(),
            }
        }

        ::std::result::Result::Ok(#ident {
            #(
                #field_idents: #temp_vars.unwrap(),
            )*
        })
    }
}
