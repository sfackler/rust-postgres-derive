use syn::{self, Ident, Type};

use crate::overrides::Overrides;

pub struct Field {
    pub name: String,
    pub ident: Ident,
    pub type_: Type,
}

impl Field {
    pub fn parse(raw: &syn::Field) -> Result<Field, String> {
        let overrides = Overrides::extract(&raw.attrs)?;

        let ident = raw.ident.as_ref().unwrap().clone();
        Ok(Field {
            name: overrides.name.unwrap_or_else(|| ident.to_string()),
            ident,
            type_: raw.ty.clone(),
        })
    }
}
