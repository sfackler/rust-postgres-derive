use syn::{Attribute, MetaItem, Lit};

pub struct Overrides {
    pub name: Option<String>,
}

impl Overrides {
    pub fn extract(attrs: &[Attribute]) -> Result<Overrides, String> {
        let mut overrides = Overrides {
            name: None,
        };

        for attr in attrs.drain(..) {
            if attr.value.name() != "postgres" {
                continue;
            }

            let list = match attr.value {
                MetaItem::List(_, ref list) => list,
                _ => return Err("expected a #[postgres(...)]".to_owned()),
            };

            for item in list {
                match *item {
                    MetaItem::NameValue(ref name, ref value) => {
                        if name != "name" {
                            return Err(format!("unknown override `{}`", name));
                        }

                        let value = match *value {
                            Lit::Str(ref s, _) => s.to_owned(),
                            _ => return Err("expected a string literal".to_owned()),
                        };

                        overrides.name = Some(value);
                    },
                    _ => return Err("expected a name-value meta item".to_owned()),
                }
            }
        }

        Ok(overrides)
    }

    pub fn strip(attrs: &mut Vec<Attribute>) {
        attrs.retain(|a| a.name() != "postgres");
    }
}
