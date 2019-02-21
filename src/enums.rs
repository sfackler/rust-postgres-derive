use syn::{self, Ident, Fields};

use crate::overrides::Overrides;

pub struct Variant {
    pub ident: Ident,
    pub name: String,
}

impl Variant {
    pub fn parse(raw: &syn::Variant) -> Result<Variant, String> {
        match raw.fields {
            Fields::Unit => {}
            _ => return Err("non-C-like enums are not supported".to_owned()),
        }

        let overrides = Overrides::extract(&raw.attrs)?;
        Ok(Variant {
            ident: raw.ident.clone(),
            name: overrides.name.unwrap_or_else(|| raw.ident.to_string()),
        })
    }
}
