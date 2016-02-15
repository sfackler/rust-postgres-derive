#![feature(rustc_private, quote)]

extern crate rustc_plugin;
extern crate syntax;

use rustc_plugin::Registry;
use syntax::ext::base::SyntaxExtension;
use syntax::feature_gate::AttributeType;
use syntax::parse::token;

mod accepts;
mod fromsql;
mod overrides;
mod tosql;

pub fn register(registry: &mut Registry) {
    registry.register_syntax_extension(
        token::intern("derive_ToSql"),
        SyntaxExtension::MultiDecorator(Box::new(tosql::expand)));
    registry.register_syntax_extension(
        token::intern("derive_FromSql"),
        SyntaxExtension::MultiDecorator(Box::new(fromsql::expand)));

    registry.register_attribute("postgres".to_owned(), AttributeType::Normal);
}
