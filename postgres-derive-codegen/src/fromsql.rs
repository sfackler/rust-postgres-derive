use syntax::ext::base::{Annotatable, ExtCtxt};
use syntax::codemap::Span;
use syntax::ast::{MetaItem, ItemKind, EnumDef, Block, VariantData, Ident};
use syntax::attr::AttrMetaMethods;
use syntax::ptr::P;
use syntax::ext::build::AstBuilder;
use syntax::parse::token;

use overrides;
use accepts;

pub fn expand(ctx: &mut ExtCtxt,
              span: Span,
              _: &MetaItem,
              annotatable: &Annotatable,
              push: &mut FnMut(Annotatable)) {
    let item = match *annotatable {
        Annotatable::Item(ref item) => item,
        _ => {
            ctx.span_err(span, "#[derive(FromSql)] can only be applied to tuple structs and enums");
            return;
        }
    };

    let overrides = overrides::get_overrides(ctx, &item.attrs);
    let name = overrides.name.unwrap_or_else(|| item.ident.name.as_str());

    let accepts_body = accepts::enum_body(ctx, name);

    let from_sql_body = match item.node {
        ItemKind::Enum(ref def, _) => enum_from_sql_body(ctx, span, item.ident, def),
        _ => {
            ctx.span_err(span, "#[derive(FromSql)] can only be applied to tuple structs and enums");
            return;
        }
    };

    let type_ = item.ident;

    let item = quote_item!(ctx,
        impl ::postgres::types::FromSql for $type_ {
            fn accepts(type_: &::postgres::types::Type) -> bool {
                $accepts_body
            }

            fn from_sql<R>(_: &::postgres::types::Type,
                           r: &mut R,
                           _: &::postgres::types::SessionInfo)
                           -> ::postgres::Result<Self>
                where R: ::std::io::Read
            {
                $from_sql_body
            }
        }
    );

    push(Annotatable::Item(item.unwrap()));
}

fn enum_from_sql_body(ctx: &mut ExtCtxt, span: Span, type_name: Ident, def: &EnumDef) -> P<Block> {
    let mut arms = vec![];

    for variant in &def.variants {
        match variant.node.data {
            VariantData::Unit(_) => {}
            _ => {
                ctx.span_err(variant.span,
                             "#[derive(FromSql)] can only be applied to C-like enums");
                continue;
            }
        }

        let variant_name = variant.node.name;
        let overrides = overrides::get_overrides(ctx, &variant.node.attrs);
        let name = overrides.name.unwrap_or_else(|| variant.node.name.name.as_str());
        arms.push(quote_arm!(ctx,
                             $name => ::std::result::Result::Ok($type_name :: $variant_name),));
    }

    arms.push(quote_arm!(ctx, v => {
        let err: ::std::boxed::Box<::std::error::Error
                                   + ::std::marker::Sync
                                   + ::std::marker::Send>
            = format!("unknown variant `{}`", v).into();
        ::std::result::Result::Err(::postgres::error::Error::Conversion(err))
    }));

    let buf = token::str_to_ident("buf");

    let match_arg = ctx.expr_addr_of(span, ctx.expr_deref(span, ctx.expr_ident(span, buf)));
    let match_ = ctx.expr_match(span, match_arg, arms);

    quote_block!(ctx, {
        let mut $buf = ::std::string::String::new();
        try!(::std::io::Read::read_to_string(r, &mut $buf));
        $match_
    })
}
