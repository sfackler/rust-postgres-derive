use std::iter;
use syn::{Ident, DeriveInput, Data, DataStruct, Fields};
use quote::Tokens;

use crate::accepts;
use crate::composites::Field;
use crate::enums::Variant;
use crate::overrides::Overrides;

pub fn expand_derive_tosql(input: DeriveInput) -> Result<Tokens, String> {
    let overrides = Overrides::extract(&input.attrs)?;

    let name = overrides.name.unwrap_or_else(|| input.ident.to_string());

    let (accepts_body, to_sql_body) = match input.data {
        Data::Enum(ref data) => {
            let variants = data.variants.iter().map(Variant::parse).collect::<Result<Vec<_>, _>>()?;
            (accepts::enum_body(&name, &variants), enum_body(&input.ident, &variants))
        }
        Data::Struct(DataStruct { fields: Fields::Unnamed(ref fields), .. }) if fields.unnamed.len() == 1 => {
            let field = fields.unnamed.first().unwrap().into_value();
            (accepts::domain_body(&name, &field), domain_body())
        }
        Data::Struct(DataStruct { fields: Fields::Named(ref fields), .. }) => {
            let fields = fields.named.iter().map(Field::parse).collect::<Result<Vec<_>, _>>()?;
            (accepts::composite_body(&name, "ToSql", None, &fields),
             composite_body(&fields))
        }
        _ => {
            return Err("#[derive(ToSql)] may only be applied to structs, single field tuple \
                        structs, and enums".to_owned())
        }
    };

    let ident = &input.ident;
    let out = quote! {
        impl ::postgres::types::ToSql for #ident {
            fn to_sql(&self,
                      _type: &::postgres::types::Type,
                      buf: &mut ::std::vec::Vec<u8>)
                      -> ::std::result::Result<::postgres::types::IsNull,
                                               ::std::boxed::Box<::std::error::Error +
                                                                 ::std::marker::Sync +
                                                                 ::std::marker::Send>> {
                #to_sql_body
            }

            fn accepts(type_: &::postgres::types::Type) -> bool {
                #accepts_body
            }

            to_sql_checked!();
        }
    };

    Ok(out)
}

fn enum_body(ident: &Ident, variants: &[Variant]) -> Tokens {
    let idents = iter::repeat(ident);
    let variant_idents = variants.iter().map(|v| &v.ident);
    let variant_names = variants.iter().map(|v| &v.name);

    quote! {
        let s = match *self {
            #(
                #idents::#variant_idents => #variant_names,
            )*
        };

        buf.extend_from_slice(s.as_bytes());
        ::std::result::Result::Ok(::postgres::types::IsNull::No)
    }
}

fn domain_body() -> Tokens {
    quote! {
        let type_ = match *_type.kind() {
            ::postgres::types::Kind::Domain(ref type_) => type_,
            _ => unreachable!(),
        };

        ::postgres::types::ToSql::to_sql(&self.0, type_, buf)
    }
}

fn composite_body(fields: &[Field]) -> Tokens {
    let field_names = fields.iter().map(|f| &f.name);
    let field_idents = fields.iter().map(|f| &f.ident);

    quote! {
        fn write_be_i32<W>(buf: &mut W, n: i32) -> ::std::io::Result<()>
            where W: ::std::io::Write
        {
            let be = [(n >> 24) as u8, (n >> 16) as u8, (n >> 8) as u8, n as u8];
            buf.write_all(&be)
        }

        let fields = match *_type.kind() {
            ::postgres::types::Kind::Composite(ref fields) => fields,
            _ => unreachable!(),
        };

        write_be_i32(buf, fields.len() as i32)?;

        for field in fields {
            write_be_i32(buf, field.type_().oid() as i32)?;

            let base = buf.len();
            write_be_i32(buf, 0)?;
            let r = match field.name() {
                #(
                    #field_names => {
                        ::postgres::types::ToSql::to_sql(&self.#field_idents,
                                                         field.type_(),
                                                         buf)
                    }
                )*
                _ => unreachable!(),
            };

            let count = match r? {
                ::postgres::types::IsNull::Yes => -1,
                ::postgres::types::IsNull::No => {
                    let len = buf.len() - base - 4;
                    if len > i32::max_value() as usize {
                        return ::std::result::Result::Err(
                            ::std::convert::Into::into("value too large to transmit"));
                    }
                    len as i32
                }
            };

            write_be_i32(&mut &mut buf[base..base + 4], count)?;
        }

        ::std::result::Result::Ok(::postgres::types::IsNull::No)
    }
}
