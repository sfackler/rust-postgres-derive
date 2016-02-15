#![feature(plugin, custom_derive)]
#![plugin(postgres_derive_macros)]

macro_rules! to_sql_checked {
    () => ()
}

#[derive(ToSql)]
enum Foo {
    Bar(i32), //~ ERROR #[derive(ToSql)] can only be applied to C-like enums
    Baz { b: i32 }, //~ ERROR #[derive(ToSql)] can only be applied to C-like enums
}

#[derive(FromSql)]
enum Foo {
    Bar(i32), //~ ERROR #[derive(FromSql)] can only be applied to C-like enums
    Baz { b: i32 }, //~ ERROR #[derive(FromSql)] can only be applied to C-like enums
}

fn main() {}
