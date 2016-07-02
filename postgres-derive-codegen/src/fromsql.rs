use syntax::ext::base::{Annotatable, ExtCtxt};
use syntax::codemap::Span;
use syntax::ast::{MetaItem, ItemKind, Block, VariantData, Ident, Ty};
use syntax::ptr::P;
use syntax::ext::build::AstBuilder;
use syntax::parse::token::{self, InternedString};

use overrides;
use accepts;
use enums;

pub fn expand(ctx: &mut ExtCtxt,
              span: Span,
              _: &MetaItem,
              annotatable: &Annotatable,
              push: &mut FnMut(Annotatable)) {
    let item = match *annotatable {
        Annotatable::Item(ref item) => item,
        _ => {
            ctx.span_err(span, "#[derive(FromSql)] can only be applied to structs, single field \
                                tuple structs, and enums");
            return;
        }
    };

    let overrides = overrides::get_overrides(ctx, &item.attrs);
    let name = overrides.name.unwrap_or_else(|| item.ident.name.as_str());

    let (accepts_body, from_sql_body) = match item.node {
        ItemKind::Enum(ref def, _) => {
            let variants = enums::get_variants(ctx, "FromSql", def);
            (accepts::enum_body(ctx, name, &variants),
             enum_from_sql_body(ctx, span, item.ident, &variants))
        }
        ItemKind::Struct(VariantData::Tuple(ref fields, _), _) => {
            if fields.len() != 1 {
                ctx.span_err(span, "#[derive(FromSql)] can only be applied to structs, single \
                                    field tuple structs, and enums");
                return;
            }
            let inner = &fields[0].ty;

            (domain_accepts_body(ctx, inner), domain_from_sql_body(ctx, item.ident, inner))
        }
        ItemKind::Struct(VariantData::Struct(ref fields, _), _) => {
            let fields = fields.iter()
                               .map(|field| {
                                   let ident = field.ident.unwrap();
                                   let overrides = overrides::get_overrides(ctx, &field.attrs);
                                   let name = overrides.name.unwrap_or_else(|| ident.name.as_str());
                                   (name, ident, &*field.ty)
                               })
                               .collect::<Vec<_>>();
            let trait_ = quote_path!(ctx, ::postgres::types::FromSql);
            (accepts::composite_body(ctx, name, &fields, &trait_),
             composite_from_sql_body(ctx, span, item.ident, &*fields))
        }
        _ => {
            ctx.span_err(span, "#[derive(FromSql)] can only be applied to structs, single field \
                                tuple structs, and enums");
            return;
        }
    };

    let type_ = item.ident;

    let item = quote_item!(ctx,
        impl ::postgres::types::FromSql for $type_ {
            fn accepts(type_: &::postgres::types::Type) -> bool {
                $accepts_body
            }

            fn from_sql<R>(_type: &::postgres::types::Type,
                           r: &mut R,
                           _info: &::postgres::types::SessionInfo)
                           -> ::postgres::Result<Self>
                where R: ::std::io::Read
            {
                $from_sql_body
            }
        }
    );

    push(Annotatable::Item(item.unwrap()));
}

fn domain_accepts_body(ctx: &mut ExtCtxt, inner: &Ty) -> P<Block> {
    quote_block!(ctx, { <$inner as ::postgres::types::FromSql>::accepts(type_) })
}

fn enum_from_sql_body(ctx: &mut ExtCtxt,
                      span: Span,
                      type_name: Ident,
                      variants: &[(Ident, InternedString)]) -> P<Block> {
    let mut arms = vec![];

    for &(ref variant_name, ref name) in variants {
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

fn domain_from_sql_body(ctx: &mut ExtCtxt, name: Ident, inner: &Ty) -> P<Block> {
    quote_block!(ctx, {
        <$inner as ::postgres::types::FromSql>::from_sql(_type, r, _info).map($name)
    })
}

fn composite_from_sql_body(ctx: &mut ExtCtxt,
                           span: Span,
                           type_name: Ident,
                           fields: &[(InternedString, Ident, &Ty)])
                           -> P<Block> {
    let mut declare_vars = vec![];
    let mut arms = vec![];
    let mut struct_fields = vec![];

    for &(ref name, ref ident, _) in fields {
        let var_name = token::str_to_ident(&format!("__{}", ident));

        declare_vars.push(quote_stmt!(ctx, let mut $var_name = ::std::option::Option::None;));

        arms.push(quote_arm!(ctx, $name => {
            $var_name = ::std::option::Option::Some(try!(read_value(len, r, field.type_(), _info)));
        }));

        struct_fields.push(ctx.field_imm(span,
                                         ident.clone(),
                                         quote_expr!(ctx, $var_name.unwrap())));
    }

    arms.push(quote_arm!(ctx, _ => unreachable!(),));
    let match_ = ctx.expr_match(span, quote_expr!(ctx, field.name()), arms);

    let build_struct = ctx.expr_struct_ident(span, type_name, struct_fields);

    quote_block!(ctx, {
        let read_be_i32 = |r: &mut R| -> ::std::io::Result<i32> {
            let mut buf = [0; 4];
            try!(::std::io::Read::read_exact(r, &mut buf));
            let num = ((buf[0] as i32) << 24)
                       | ((buf[1] as i32) << 16)
                       | ((buf[2] as i32) << 8)
                       | (buf[3] as i32);
            ::std::result::Result::Ok(num)
        };

        fn read_value<R, T>(len: i32,
                            r: &mut R,
                            type_: &::postgres::types::Type,
                            info: &::postgres::types::SessionInfo)
                            -> ::postgres::Result<T>
            where R: ::std::io::Read,
                  T: ::postgres::types::FromSql
        {
            if len < 0 {
                ::postgres::types::FromSql::from_sql_null(type_, info)
            } else {
                let mut r = ::std::io::Read::take(::std::io::Read::by_ref(r), len as u64);
                ::postgres::types::FromSql::from_sql(type_, &mut r, info)
            }
        }

        let fields = match _type.kind() {
            &::postgres::types::Kind::Composite(ref fields) => fields,
            _ => unreachable!(),
        };

        let num_fields = try!(read_be_i32(r));
        if num_fields as usize != fields.len() {
            let err: ::std::boxed::Box<::std::error::Error
                                       + ::std::marker::Sync
                                       + ::std::marker::Send>
                = format!("expected {} fields but saw {}", fields.len(), num_fields).into();
            return ::std::result::Result::Err(::postgres::error::Error::Conversion(err))
        }

        $declare_vars;

        for field in fields {
            let oid = try!(read_be_i32(r)) as u32;
            if oid != field.type_().oid() {
                let err: ::std::boxed::Box<::std::error::Error
                                           + ::std::marker::Sync
                                           + ::std::marker::Send>
                    = format!("expected OID {} but saw {}", field.type_().oid(), oid).into();
                return ::std::result::Result::Err(::postgres::error::Error::Conversion(err))
            }

            let len = try!(read_be_i32(r));

            $match_
        }

        ::std::result::Result::Ok($build_struct)
    })
}
