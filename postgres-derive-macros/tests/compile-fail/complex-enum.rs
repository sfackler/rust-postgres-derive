#![feature(plugin, custom_derive)]
#![plugin(postgres_derive_macros)]

macro_rules! to_sql_checked {
    () => ()
}

#[derive(ToSql)]
enum Foo {
    Bar(i32), //~ ERROR #[derive(ToSql)] does not support non-C-like enums
    Baz { b: i32 }, //~ ERROR #[derive(ToSql)] does not support non-C-like enums
}

#[derive(FromSql)]
enum Foo {
    Bar(i32), //~ ERROR #[derive(FromSql)] does not support non-C-like enums
    Baz { b: i32 }, //~ ERROR #[derive(FromSql)] does not support non-C-like enums
}

fn main() {}
