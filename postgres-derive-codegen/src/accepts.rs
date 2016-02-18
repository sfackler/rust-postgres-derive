use syntax::ext::base::ExtCtxt;
use syntax::ast::{Block, Ty, Path, Ident};
use syntax::ptr::P;
use syntax::parse::token::InternedString;

pub fn enum_body(ctx: &mut ExtCtxt,
                 name: InternedString,
                 variants: &[(Ident, InternedString)])
                 -> P<Block> {
    let num_variants = variants.len();

    let arms = variants.iter()
                       .map(|&(_, ref variant)| quote_arm!(ctx, $variant => true,))
                       .collect::<Vec<_>>();

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
                        _ => false,
                    }
                })
            }
            _ => false
        }
    })
}

pub fn composite_body(ctx: &mut ExtCtxt,
                      name: InternedString,
                      fields: &[(InternedString, Ident, &Ty)],
                      trait_: &Path)
                      -> P<Block> {
    let num_fields = fields.len();

    let arms = fields.iter()
                     .map(|&(ref name, _, ty)| {
                         quote_arm!(ctx, $name => <$ty as $trait_>::accepts(field.type_()),)
                     })
                     .collect::<Vec<_>>();

    quote_block!(ctx, {
        if type_.name() != $name {
            return false;
        }

        match type_.kind() {
            &::postgres::types::Kind::Composite(ref fields) => {
                if $num_fields != fields.len() {
                    return false;
                }

                fields.iter().all(|field| {
                    match field.name() {
                        $arms
                        _ => false
                    }
                })
            }
            _ => false
        }
    })
}
