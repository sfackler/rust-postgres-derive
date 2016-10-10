use std::iter;
use syn::{self, Body, Ident, MacroInput, VariantData};
use quote::Tokens;

use accepts;
use composites::Field;
use enums::Variant;
use overrides::Overrides;

pub fn expand_derive_fromsql(input: &MacroInput) -> Result<String, String> {
    let overrides = try!(Overrides::extract(&input.attrs));

    let name = overrides.name.unwrap_or_else(|| input.ident.to_string());

    let (accepts_body, to_sql_body) = match input.body {
        Body::Enum(ref variants) => {
            let variants: Vec<Variant> = try!(variants.iter().map(Variant::parse).collect());
            (accepts::enum_body(&name, &variants), enum_body(&input.ident, &variants))
        }
        Body::Struct(VariantData::Tuple(ref fields)) if fields.len() == 1 => {
            let field = &fields[0];
            (domain_accepts_body(field), domain_body(&input.ident, field))
        }
        Body::Struct(VariantData::Struct(ref fields)) => {
            let fields: Vec<Field> = try!(fields.iter().map(Field::parse).collect());
            (accepts::composite_body(&name, "FromSql", &fields),
             composite_body(&input.ident, &fields))
        }
        _ => {
            return Err("#[derive(ToSql)] may only be applied to structs, single field tuple \
                        structs, and enums".to_owned())
        }
    };

    let ident = &input.ident;
    let out = quote! {
        impl ::postgres::types::FromSql for #ident {
            fn from_sql(_type: &::postgres::types::Type,
                        buf: &[u8],
                        _info: &::postgres::types::SessionInfo)
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

    Ok(out.to_string())
}

fn enum_body(ident: &Ident, variants: &[Variant]) -> Tokens {
    let variant_names = variants.iter().map(|v| &v.name);
    let idents = iter::repeat(ident);
    let variant_idents = variants.iter().map(|v| &v.ident);

    quote! {
        match try!(::std::str::from_utf8(buf)) {
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

fn domain_accepts_body(field: &syn::Field) -> Tokens {
    let ty = &field.ty;
    quote! {
        <#ty as ::postgres::types::FromSql>::accepts(type_)
    }
}

fn domain_body(ident: &Ident, field: &syn::Field) -> Tokens {
    let ty = &field.ty;
    quote! {
        <#ty as ::postgres::types::FromSql>::from_sql(_type, buf, _info).map(#ident)
    }
}

fn composite_body(ident: &Ident, fields: &[Field]) -> Tokens {
    let temp_vars = &fields.iter().map(|f| Ident::new(format!("__{}", f.ident))).collect::<Vec<_>>();
    let field_names = &fields.iter().map(|f| &f.name).collect::<Vec<_>>();
    let field_idents = &fields.iter().map(|f| &f.ident).collect::<Vec<_>>();

    quote! {
        fn read_be_i32(buf: &mut &[u8]) -> ::std::io::Result<i32> {
            let mut bytes = [0; 4];
            try!(::std::io::Read::read_exact(buf, &mut bytes));
            let num = ((bytes[0] as i32) << 24) |
                ((bytes[1] as i32) << 16) |
                ((bytes[2] as i32) << 8) |
                (bytes[3] as i32);
            ::std::result::Result::Ok(num)
        }

        fn read_value<T>(type_: &::postgres::types::Type,
                         buf: &mut &[u8],
                         info: &::postgres::types::SessionInfo)
                         -> ::std::result::Result<T,
                             ::std::boxed::Box<::std::error::Error +
                             ::std::marker::Sync +
                             ::std::marker::Send>>
                         where T: ::postgres::types::FromSql
        {
            let len = try!(read_be_i32(buf));
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
            ::postgres::types::FromSql::from_sql_nullable(type_, value, info)
        }

        let fields = match *_type.kind() {
            ::postgres::types::Kind::Composite(ref fields) => fields,
            _ => unreachable!(),
        };

        let mut buf = buf;
        let num_fields = try!(read_be_i32(&mut buf));
        if num_fields as usize != fields.len() {
            return ::std::result::Result::Err(
                ::std::convert::Into::into(format!("invalid field count: {} vs {}", num_fields,
                                                   fields.len())));
        }

        #(
            let mut #temp_vars = ::std::option::Option::None;
        )*

        for field in fields {
            let oid = try!(read_be_i32(&mut buf)) as u32;
            if oid != field.type_().oid() {
                return ::std::result::Result::Err(::std::convert::Into::into("unexpected OID"));
            }

            match field.name() {
                #(
                    #field_names => {
                        #temp_vars = ::std::option::Option::Some(
                            try!(read_value(field.type_(), &mut buf, _info)));
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
