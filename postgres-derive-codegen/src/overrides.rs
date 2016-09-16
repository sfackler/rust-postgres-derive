use syntax::ast::{Attribute, LitKind, MetaItemKind, NestedMetaItemKind};
use syntax::ext::base::ExtCtxt;
use syntax::parse::token::InternedString;

pub struct Overrides {
    pub name: Option<InternedString>,
}

pub fn get_overrides(ctx: &mut ExtCtxt, attrs: &[Attribute]) -> Overrides {
    let mut overrides = Overrides { name: None };

    for attr in attrs {
        if attr.check_name("postgres") {
            let list = match attr.meta_item_list() {
                Some(list) => list,
                None => {
                    ctx.span_err(attr.span, "expected #[postgres(...)]");
                    continue;
                }
            };

            for item in list {
                let item = match item.node {
                    NestedMetaItemKind::MetaItem(ref item) => item,
                    _ => {
                        ctx.span_err(item.span, "expected a meta item");
                        continue;
                    }
                };

                match item.node {
                    MetaItemKind::NameValue(ref key, ref value) => {
                        if *key != "name" {
                            ctx.span_err(item.span, &format!("unknown attribute key `{}`", key));
                            continue;
                        }

                        match value.node {
                            LitKind::Str(ref s, _) => overrides.name = Some(s.clone()),
                            _ => {
                                ctx.span_err(value.span, "expected string literal");
                                continue;
                            }
                        }
                    }
                    _ => {
                        ctx.span_err(item.span, "expected a key-value meta item");
                        continue;
                    }
                }
            }
        }
    }

    overrides
}
