use std::fmt::Write;
use quote::{Tokens, ToTokens};

use enums::Variant;
use composites::Field;

pub fn enum_body(name: &str, variants: &[Variant]) -> String {
    let mut body = String::new();

    write!(body, "
        if type_.name() != \"{}\" {{
            return false;
        }}

        match *type_.kind() {{
            ::postgres::types::Kind::Enum(ref variants) => {{
                if variants.len() != {} {{
                    return false;
                }}

                variants.iter().all(|v| {{
                    match &**v {{", name, variants.len()).unwrap();

    for variant in variants {
        write!(body, "
                        \"{}\" => true,", variant.name).unwrap();
    }

    write!(body, "
                        _ => false,
                    }}
                }})
            }}
            _ => false,
        }}").unwrap();

    body
}

pub fn composite_body(name: &str, trait_: &str, fields: &[Field]) -> String {
    let mut body = String::new();

    write!(body, "
        if type_.name() != \"{}\" {{
            return false;
        }}

        match *type_.kind() {{
            ::postgres::types::Kind::Composite(ref fields) => {{
                if fields.len() != {} {{
                    return false;
                }}

                fields.iter().all(|f| {{
                    match f.name() {{", name, fields.len()).unwrap();

    for field in fields {
        let mut tokens = Tokens::new();
        field.type_.to_tokens(&mut tokens);
        write!(body, "
                        \"{}\" => <{} as ::postgres::types::{}>::accepts(f.type_()),",
               field.name, tokens, trait_).unwrap();
    }

    write!(body, "\
                        _ => false,\
                    }}\
                }})\
            }}\
            _ => false,\
        }}").unwrap();

    body
}
