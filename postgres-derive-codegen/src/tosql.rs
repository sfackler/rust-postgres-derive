use syntax::ext::base::{Annotatable, ExtCtxt};
use syntax::codemap::Span;
use syntax::ast::{MetaItem, ItemKind, EnumDef, Block, VariantData, Ident};
use syntax::attr::AttrMetaMethods;
use syntax::ptr::P;
use syntax::ext::build::AstBuilder;

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
            ctx.span_err(span, "#[derive(ToSql)] can only be applied to tuple structs and enums");
            return;
        }
    };

    let overrides = overrides::get_overrides(ctx, &item.attrs);
    let name = overrides.name.unwrap_or_else(|| item.ident.name.as_str());

    let accepts_body = accepts::enum_body(ctx, name);

    let (to_sql_body, generics) = match item.node {
        ItemKind::Enum(ref def, ref generics) => {
            (enum_to_sql_body(ctx, span, item.ident, def), generics)
        }
        _ => {
            ctx.span_err(span, "#[derive(ToSql)] can only be applied to tuple structs and enums");
            return;
        }
    };

    let type_ = item.ident;
    let where_clause = &generics.where_clause;

    let item = quote_item!(ctx,
        impl ::postgres::types::ToSql for $type_ {
            to_sql_checked!();

            fn accepts(type_: &::postgres::types::Type) -> bool {
                $accepts_body
            }

            fn to_sql<W: ?Sized>(&self,
                                 type_: &::postgres::types::Type,
                                 out: &mut W,
                                 _: &::postgres::types::SessionInfo)
                                 -> ::postgres::Result<::postgres::types::IsNull>
                where W: ::std::io::Write
            {
                $to_sql_body
            }
        }
    );

    push(Annotatable::Item(item.unwrap()));
}

fn enum_to_sql_body(ctx: &mut ExtCtxt, span: Span, type_name: Ident, def: &EnumDef) -> P<Block> {
    let mut arms = vec![];

    for variant in &def.variants {
        match variant.node.data {
            VariantData::Unit(_) => {}
            _ => {
                ctx.span_err(variant.span, "#[derive(ToSql)] can only be applied to C-like enums");
                continue;
            }
        }

        let variant_name = variant.node.name;
        let overrides = overrides::get_overrides(ctx, &variant.node.attrs);
        let name = overrides.name.unwrap_or_else(|| variant.node.name.name.as_str());
        arms.push(quote_arm!(ctx, $type_name :: $variant_name => $name,));
    }

    let match_arg = ctx.expr_deref(span, ctx.expr_self(span));
    let match_ = ctx.expr_match(span, match_arg, arms);

    quote_block!(ctx, {
        let s: &'static str = $match_;
        try!(out.write_all(s.as_bytes()));
        Ok(::postgres::types::IsNull::Yes)
    })
}
