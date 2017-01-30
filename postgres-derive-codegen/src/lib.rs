extern crate syntex;
extern crate syntex_syntax as syntax;
extern crate postgres_derive_internals;

use syntax::ext::base::{Annotatable, ExtCtxt};
use syntax::codemap::Span;
use syntax::ast::MetaItem;
use syntax::print::pprust;
use syntax::parse;

pub fn expand<S, D>(src: S, dst: D) -> Result<(), syntex::Error>
    where S: AsRef<std::path::Path>,
          D: AsRef<std::path::Path>,
{
    let mut registry = syntex::Registry::new();
    register(&mut registry);
    registry.expand("", src.as_ref(), dst.as_ref())
}

pub fn register(reg: &mut syntex::Registry) {
    use syntax::{ast, fold};

    fn strip_attributes(krate: ast::Crate) -> ast::Crate {
        struct StripAttributeFolder;

        impl fold::Folder for StripAttributeFolder {
            fn fold_attribute(&mut self, attr: ast::Attribute) -> Option<ast::Attribute> {
                if attr.value.name == "postgres" {
                    return None;
                }

                Some(attr)
            }

            fn fold_mac(&mut self, mac: ast::Mac) -> ast::Mac {
                fold::noop_fold_mac(mac, self)
            }
        }

        fold::Folder::fold_crate(&mut StripAttributeFolder, krate)
    }

    reg.add_attr("feature(custom_derive)");
    reg.add_attr("feature(custom_attribute)");

    reg.add_decorator("derive_ToSql", expand_tosql);
    reg.add_decorator("derive_FromSql", expand_fromsql);

    reg.add_post_expansion_pass(strip_attributes);
}

fn expand_tosql(ctx: &mut ExtCtxt,
                span: Span,
                _: &MetaItem,
                annotatable: &Annotatable,
                push: &mut FnMut(Annotatable)) {
    expand_inner(ctx,
                 span,
                 "ToSql",
                 annotatable,
                 push,
                 postgres_derive_internals::expand_derive_tosql);
}

fn expand_fromsql(ctx: &mut ExtCtxt,
                  span: Span,
                  _: &MetaItem,
                  annotatable: &Annotatable,
                  push: &mut FnMut(Annotatable)) {
    expand_inner(ctx,
                 span,
                 "FromSql",
                 annotatable,
                 push,
                 postgres_derive_internals::expand_derive_fromsql);
}

fn expand_inner(ctx: &mut ExtCtxt,
                span: Span,
                trait_: &str,
                annotatable: &Annotatable,
                push: &mut FnMut(Annotatable),
                expand: fn(&str) -> Result<String, String>) {
    let item = match *annotatable {
        Annotatable::Item(ref item) => item,
        _ => {
            ctx.span_err(span, &format!("#[derive({})] can only be applied to structs, single field \
                                         tuple structs, and enums", trait_));
            return;
        }
    };

    let item = pprust::item_to_string(item);
    match expand(&item) {
        Ok(source) => {
            let mut parser = parse::new_parser_from_source_str(&ctx.parse_sess,
                                                               "<macro src>".to_owned(),
                                                               source);
            while let Some(item) = parser.parse_item().unwrap() {
                push(Annotatable::Item(item));
            }
        }
        Err(err) => ctx.span_err(span, &err),
    }
}
