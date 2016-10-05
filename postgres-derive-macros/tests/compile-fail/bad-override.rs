#![feature(plugin, custom_derive)]
#![plugin(postgres_derive_macros)]

macro_rules! to_sql_checked {
    () => ()
}

#[derive(Clone, ToSql)]
#[postgres(
    foo = "bar" //~ ERROR unknown attribute key `foo`
)]
enum Foo {
    #[postgres] //~ ERROR expected #[postgres(...)]
    Bar,
    #[postgres(
        name //~ ERROR expected a key-value meta item
    )]
    Baz,
}

fn main() {}
