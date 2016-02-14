use syntax::ext::base::{Annotatable, ExtCtxt};
use syntax::codemap::Span;
use syntax::ast::{MetaItem, ItemKind, EnumDef, Generics};

pub fn expand(cx: &mut ExtCtxt,
              span: Span,
              meta_item: &MetaItem,
              annotatable: &Annotatable,
              push: &mut FnMut(Annotatable)) {
    let item = match *annotatable {
        Annotatable::Item(ref item) => item,
        _ => {
            cx.span_err(span, "#[derive(ToSql)] can only be applied to tuple structs and enums");
            return;
        }
    };

    match item.node {
        ItemKind::Enum(ref def, ref generics) => expand_enum(cx, span, def, generics, push),
        _ => {
            cx.span_err(span, "#[derive(ToSql)] can only be applied to tuple structs and enums");
            return;
        }
    }
}

fn expand_enum(cx: &mut ExtCtxt,
               span: Span,
               def: &EnumDef,
               generics: &Generics,
               push: &mut FnMut(Annotatable)) {
}
