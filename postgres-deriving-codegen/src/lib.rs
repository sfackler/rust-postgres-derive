#![feature(rustc_private)]

extern crate rustc_plugin;
extern crate syntax;

use rustc_plugin::Registry;
use syntax::ext::base::SyntaxExtension;
use syntax::feature_gate::AttributeType;
use syntax::parse::token;

mod enums;

pub fn register(registry: &mut Registry) {
    registry.register_syntax_extension(
        token::intern("derive_PgEnum"),
        SyntaxExtension::MultiDecorator(Box::new(enums::expand_derive_pgenum)));

    registry.register_attribute("pg".to_owned(), AttributeType::Normal);
}
