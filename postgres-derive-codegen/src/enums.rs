use syntax::ext::base::ExtCtxt;
use syntax::ast::{Ident, EnumDef, VariantData};
use syntax::parse::token::InternedString;

use overrides;

pub fn get_variants(ctx: &mut ExtCtxt, trait_: &str, def: &EnumDef) -> Vec<(Ident, InternedString)> {
    let mut variants = vec![];

    for variant in &def.variants {
        match variant.node.data {
            VariantData::Unit(_) => {}
            _ => {
                ctx.span_err(variant.span,
                             &format!("#[derive({})] does not support non-C-like enums", trait_));
                continue;
            }
        }

        let variant_name = variant.node.name;
        let overrides = overrides::get_overrides(ctx, &variant.node.attrs);
        let name = overrides.name.unwrap_or_else(|| variant_name.name.as_str());
        variants.push((variant_name, name));
    }

    variants
}
