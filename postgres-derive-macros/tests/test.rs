#![feature(plugin, custom_derive)]
#![plugin(postgres_derive_macros)]

#[macro_use]
extern crate postgres;

#[derive(Debug, ToSql)]
enum Foo {
    Bar,
    Baz
}

#[derive(Debug, ToSql)]
#[postgres(name = "mood")]
enum Mood {
    #[postgres(name = "sad")]
    Sad,
    #[postgres(name = "ok")]
    Ok,
    #[postgres(name = "happy")]
    Happy,
}
