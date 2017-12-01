use syn::{Attribute, MetaItem, NestedMetaItem, Lit};

pub struct Overrides {
    pub name: Option<String>,
    pub schema: Option<String>,
}

impl Overrides {
    pub fn extract(attrs: &[Attribute]) -> Result<Overrides, String> {
        let mut overrides = Overrides {
            name: None,
            schema: None,
        };

        for attr in attrs {
            if attr.value.name() != "postgres" {
                continue;
            }

            let list = match attr.value {
                MetaItem::List(_, ref list) => list,
                _ => return Err("expected a #[postgres(...)]".to_owned()),
            };

            for item in list {
                match *item {
                    NestedMetaItem::MetaItem(MetaItem::NameValue(ref name, ref value)) => {

                        let value = match *value {
                            Lit::Str(ref s, _) => s.to_owned(),
                            _ => return Err("expected a string literal".to_owned()),
                        };

                        match name.as_ref() {
                            "name" => overrides.name = Some(value),
                            "schema" => overrides.schema = Some(value),
                            name => return Err(format!("unknown override `{}`", name)),
                        }
                    },
                    _ => return Err("expected a name-value meta item".to_owned()),
                }
            }
        }

        Ok(overrides)
    }
}
