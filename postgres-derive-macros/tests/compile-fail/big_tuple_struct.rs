#![feature(plugin, custom_derive)]
#![plugin(postgres_derive_macros)]

macro_rules! to_sql_checked {
    () => ()
}

#[derive(ToSql)] //~ ERROR #[derive(ToSql)] can only be applied to one field tuple structs
struct Foo(i32, i32);

#[derive(FromSql)] //~ ERROR #[derive(FromSql)] can only be applied to one field tuple structs
struct Bar(i32, i32);

fn main() {}
