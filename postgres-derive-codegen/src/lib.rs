#![cfg_attr(not(feature = "with-syntex"), feature(rustc_private, quote))]

#[cfg(feature = "with-syntex")]
extern crate syntex;
#[cfg(feature = "with-syntex")]
extern crate syntex_syntax as syntax;
#[cfg(not(feature = "with-syntex"))]
extern crate syntax;
#[cfg(not(feature = "with-syntex"))]
extern crate rustc_plugin;

#[cfg(feature = "with-syntex")]
include!(concat!(env!("OUT_DIR"), "/lib.rs"));

#[cfg(not(feature = "with-syntex"))]
include!("lib.rs.in");

#[cfg(feature = "with-syntex")]
pub fn expand<S, D>(src: S, dst: D) -> Result<(), syntex::Error>
    where S: AsRef<std::path::Path>,
          D: AsRef<std::path::Path>,
{
    let mut registry = syntex::Registry::new();
    register(&mut registry);
    registry.expand("", src.as_ref(), dst.as_ref())
}

#[cfg(feature = "with-syntex")]
pub fn register(reg: &mut syntex::Registry) {
    use syntax::{ast, fold};

    fn strip_attributes(krate: ast::Crate) -> ast::Crate {
        struct StripAttributeFolder;

        impl fold::Folder for StripAttributeFolder {
            fn fold_attribute(&mut self, attr: ast::Attribute) -> Option<ast::Attribute> {
                match attr.node.value.node {
                    ast::MetaItemKind::List(ref n, _) if n == &"postgres" => return None,
                    _ => {}
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

    reg.add_decorator("derive_ToSql", tosql::expand);
    reg.add_decorator("derive_FromSql", fromsql::expand);

    reg.add_post_expansion_pass(strip_attributes);
}

#[cfg(not(feature = "with-syntex"))]
pub fn register(registry: &mut rustc_plugin::Registry) {
    use syntax::ext::base::SyntaxExtension;
    use syntax::feature_gate::AttributeType;
    use syntax::parse::token;

    registry.register_syntax_extension(token::intern("derive_ToSql"),
                                       SyntaxExtension::MultiDecorator(Box::new(tosql::expand)));
    registry.register_syntax_extension(token::intern("derive_FromSql"),
                                       SyntaxExtension::MultiDecorator(Box::new(fromsql::expand)));

    registry.register_attribute("postgres".to_owned(), AttributeType::Normal);
}
