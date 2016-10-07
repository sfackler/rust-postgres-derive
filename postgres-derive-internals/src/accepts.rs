use std::fmt::Write;

use enums::Variant;

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
