use syntax::ext::base::{Annotatable, ExtCtxt};
use syntax::codemap::Span;
use syntax::ast::MetaItem;

pub fn expand_derive_pgenum(cx: &mut ExtCtxt,
                            span: Span,
                            meta_item: &MetaItem,
                            annotatable: &Annotatable,
                            push: &mut FnMut(Annotatable)) {
}
