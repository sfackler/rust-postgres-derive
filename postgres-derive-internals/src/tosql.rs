use std::fmt::Write;
use syn::{Body, Ident, MacroInput};

use accepts;
use enums::Variant;
use overrides::Overrides;

pub fn expand_derive_tosql(input: &MacroInput) -> Result<String, String> {
    let overrides = try!(Overrides::extract(&input.attrs));

    let name = overrides.name.unwrap_or_else(|| input.ident.to_string());

    let (accepts_body, to_sql_body) = match input.body {
        Body::Enum(ref variants) => {
            let variants: Vec<Variant> = try!(variants.iter().map(Variant::parse).collect());
            (accepts::enum_body(&variants), enum_body(&input.ident, &variants))
        }
        _ => {
            return Err("#[derive(ToSql)] may only be applied to structs, single field tuple \
                        structs, and enums".to_owned())
        }
    };

    let out = format!("
impl ::postgres::types::ToSql for {} {{
    fn to_sql(&self,
              _: &::postgres::types::Type,
              buf: &mut ::std::vec::Vec<u8>,
              _: &::postgres::types::SessionInfo)
              -> ::std::result::Result<::postgres::types::IsNull,
                                       ::std::boxed::Box<::std::error::Error +
                                                         ::std::marker::Sync +
                                                         ::std::marker::Send>> {{{}
    }}

    fn accepts(type_: &::postgres::types::Type) -> bool {{
        if type_.name() != \"{}\" {{
            return false;
        }}
{}
    }}

    to_sql_checked!();
}}", input.ident, to_sql_body, name, accepts_body);

    Ok(out)
}

fn enum_body(ident: &Ident, variants: &[Variant]) -> String {
    let mut out = "
        let s = match *self {".to_owned();

    for variant in variants {
        write!(out, "
            {}::{} => \"{}\",", ident, variant.ident, variant.name).unwrap();
    }

    out.push_str("
        };

        buf.extend_from_slice(s.as_bytes());
        Ok(::postgres::types::IsNull::No)");

    out
}
