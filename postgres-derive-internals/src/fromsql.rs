use std::fmt::Write;
use syn::{Body, Ident, MacroInput, VariantData, Field};
use quote::{Tokens, ToTokens};

use accepts;
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

fn domain_accepts_body(field: &Field) -> String {
    let mut tokens = Tokens::new();
    field.ty.to_tokens(&mut tokens);
    format!("
        <{} as ::postgres::types::FromSql>::accepts(type_)", tokens)
}

fn domain_body(ident: &Ident, field: &Field) -> String {
    let mut tokens = Tokens::new();
    field.ty.to_tokens(&mut tokens);
    format!("\
        <{} as ::postgres::types::FromSql>::from_sql(_type, buf, _info).map({})", tokens, ident)
}
