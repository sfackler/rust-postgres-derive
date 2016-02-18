use syntax::ext::base::ExtCtxt;
use syntax::ext::build::AstBuilder;
use syntax::ast::{Block, Ty, Path, Ident};
use syntax::codemap::Span;
use syntax::ptr::P;
use syntax::parse::token::InternedString;

pub fn enum_body(ctx: &mut ExtCtxt,
                 name: InternedString,
                 variants: &[(Ident, InternedString)])
                 -> P<Block> {
    let num_variants = variants.len();

    let mut arms = variants.iter()
                           .map(|&(_, ref variant)| quote_arm!(ctx, $variant => true,))
                           .collect::<Vec<_>>();
    arms.push(quote_arm!(ctx, _ => false,));

    quote_block!(ctx, {
        if type_.name() != $name {
            return false;
        }

        match type_.kind() {
            &::postgres::types::Kind::Enum(ref variants) => {
                if variants.len() != $num_variants {
                    return false;
                }

                variants.iter().all(|variant| {
                    match &**variant {
                        $arms
                    }
                })
            }
            _ => false
        }
    })
}

pub fn composite_body(ctx: &mut ExtCtxt,
                      span: Span,
                      name: InternedString,
                      fields: &[(InternedString, Ident, &Ty)],
                      trait_: &Path)
                      -> P<Block> {
    let num_fields = fields.len();

    let mut arms = fields.iter()
                         .map(|&(ref name, _, ty)| {
                             quote_arm!(ctx, $name => {
                                if !<$ty as $trait_>::accepts(field.type_()) {
                                    return false;
                                }
                             })
                         })
                         .collect::<Vec<_>>();
    arms.push(quote_arm!(ctx, _ => return false,));
    let match_ = ctx.expr_match(span, quote_expr!(ctx, field.name()), arms);

    quote_block!(ctx, {
        if type_.name() != $name {
            return false;
        }

        match type_.kind() {
            &::postgres::types::Kind::Composite(ref fields) => {
                if $num_fields != fields.len() {
                    return false;
                }

                for field in fields {
                    $match_
                }

                true
            }
            _ => false
        }
    })
}
