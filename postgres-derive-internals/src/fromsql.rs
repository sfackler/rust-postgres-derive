use std::fmt::Write;
use syn::{self, Body, Ident, MacroInput, VariantData};
use quote::{Tokens, ToTokens};

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

    let out = format!("
impl ::postgres::types::FromSql for {} {{
    fn from_sql(_type: &::postgres::types::Type,
                buf: &[u8],
                _info: &::postgres::types::SessionInfo)
                -> ::std::result::Result<{},
                                         ::std::boxed::Box<::std::error::Error +
                                                           ::std::marker::Sync +
                                                           ::std::marker::Send>> {{{}
    }}

    fn accepts(type_: &::postgres::types::Type) -> bool {{{}
    }}
}}", input.ident, input.ident, to_sql_body, accepts_body);

    Ok(out)
}

fn enum_body(ident: &Ident, variants: &[Variant]) -> String {
    let mut out = "
        match buf {".to_owned();

    for variant in variants {
        write!(out, "
            b\"{}\" => Ok({}::{}),", variant.name, ident, variant.ident).unwrap();
    }

    out.push_str("
            s => {
                ::std::result::Result::Err(
                    ::std::convert::Into::into(format!(\"invalid variant `{}`\",
                                               ::std::string::String::from_utf8_lossy(s))))
            }
        }");

    out
}

fn domain_accepts_body(field: &syn::Field) -> String {
    let mut tokens = Tokens::new();
    field.ty.to_tokens(&mut tokens);
    format!("
        <{} as ::postgres::types::FromSql>::accepts(type_)", tokens)
}

fn domain_body(ident: &Ident, field: &syn::Field) -> String {
    let mut tokens = Tokens::new();
    field.ty.to_tokens(&mut tokens);
    format!("\
        <{} as ::postgres::types::FromSql>::from_sql(_type, buf, _info).map({})", tokens, ident)
}

fn composite_body(ident: &Ident, fields: &[Field]) -> String {
    let mut out = "
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
                        ::std::convert::Into::into(\"invalid buffer size\"));
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
                ::std::convert::Into::into(format!(\"invalid field count: {} vs {}\", num_fields,
                                                   fields.len())));
        }
        ".to_owned();

    for field in fields {
        write!(out, "
        let mut __{} = ::std::option::Option::None;", field.ident).unwrap();
    }

    write!(out, "
        for field in fields {{
            let oid = try!(read_be_i32(&mut buf)) as u32;
            if oid != field.type_().oid() {{
                return ::std::result::Result::Err(
                    ::std::convert::Into::into(\"unexpected OID\"));
            }}\
            \
            match field.name() {{").unwrap();

    for field in fields {
        write!(out, "
                \"{}\" => {{
                    __{} = ::std::option::Option::Some(
                        try!(read_value(field.type_(), &mut buf, _info)));
                }}",
               field.name, field.ident).unwrap();
    }

    write!(out, "
                _ => unreachable!(),
            }}
        }}

        ::std::result::Result::Ok({} {{", ident).unwrap();

    for field in fields {
        write!(out, "
            {}: __{}.unwrap(),", field.ident, field.ident).unwrap();
    }

    write!(out, "
        }})").unwrap();

    out
}
