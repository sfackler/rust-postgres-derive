use syn::{Attribute, Meta, NestedMeta, Lit};

pub struct Overrides {
    pub name: Option<String>,
    pub transparent: bool,
}

impl Overrides {
    pub fn extract(attrs: &[Attribute]) -> Result<Overrides, String> {
        let mut overrides = Overrides {
            name: None,
            transparent: false,
        };

        for attr in attrs {
            let attr = match attr.interpret_meta() {
                Some(meta) => meta,
                None => continue,
            };

            if attr.name() != "postgres" {
                continue;
            }

            let list = match attr {
                Meta::List(ref list) => list,
                _ => return Err("expected a #[postgres(...)]".to_owned()),
            };

            for item in &list.nested {
                match *item {
                    NestedMeta::Meta(Meta::NameValue(ref meta)) => {
                        if meta.ident.as_ref() != "name" {
                            return Err(format!("unknown override `{}`", meta.ident.as_ref()));
                        }

                        let value = match meta.lit {
                            Lit::Str(ref s) => s.value(),
                            _ => return Err("expected a string literal".to_owned()),
                        };

                        overrides.name = Some(value);
                    },
                    NestedMeta::Meta(Meta::Word(ref meta)) => {
                        if meta.as_ref() == "transparent" {
                            overrides.transparent = true;
                        } else {
                            return Err(format!("unknown override `{}`", meta.as_ref()));
                        }
                    }
                    _ => return Err("expected a name-value or word meta item".to_owned()),
                }
            }
        }

        if overrides.name.is_some() && overrides.transparent {
            return Err("overrides `name` and `transparent` may not be used at the same time".to_owned())
        }

        Ok(overrides)
    }
}
