use syn::{self, Ident, Ty};

use overrides::Overrides;

pub struct Field {
    pub name: String,
    pub ident: Ident,
    pub type_: Ty,
}

impl Field {
    pub fn parse(raw: &syn::Field) -> Result<Field, String> {
        let overrides = Overrides::extract(&raw.attrs)?;

        let ident = raw.ident.as_ref().unwrap().clone();
        Ok(Field {
            name: overrides.name.unwrap_or_else(|| ident.to_string()),
            ident: ident,
            type_: raw.ty.clone(),
        })
    }
}
