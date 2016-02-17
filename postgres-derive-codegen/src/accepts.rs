use syntax::ext::base::ExtCtxt;
use syntax::ast::Block;
use syntax::ptr::P;
use syntax::parse::token::InternedString;

pub fn enum_body(ctx: &mut ExtCtxt, name: InternedString) -> P<Block> {
    quote_block!(ctx, {
        type_.name() == $name && type_.kind() == &::postgres::types::Kind::Enum
    })
}
