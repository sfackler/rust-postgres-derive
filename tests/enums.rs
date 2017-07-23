#[macro_use]
extern crate postgres_derive;
#[macro_use]
extern crate postgres;

use postgres::{Connection, TlsMode};
use postgres::types::WrongType;

mod util;

#[test]
fn defaults() {
    #[derive(Debug, ToSql, FromSql, PartialEq)]
    enum Foo {
        Bar,
        Baz
    }

    let conn = Connection::connect("postgres://postgres@localhost", TlsMode::None).unwrap();
    conn.execute("CREATE TYPE pg_temp.\"Foo\" AS ENUM ('Bar', 'Baz')", &[]).unwrap();

    util::test_type(&conn, "\"Foo\"", &[(Foo::Bar, "'Bar'"), (Foo::Baz, "'Baz'")]);
}

#[test]
fn name_overrides() {
    #[derive(Debug, ToSql, FromSql, PartialEq)]
    #[postgres(name = "mood")]
    enum Mood {
        #[postgres(name = "sad")]
        Sad,
        #[postgres(name = "ok")]
        Ok,
        #[postgres(name = "happy")]
        Happy,
    }

    let conn = Connection::connect("postgres://postgres@localhost", TlsMode::None).unwrap();
    conn.execute("CREATE TYPE pg_temp.mood AS ENUM ('sad', 'ok', 'happy')", &[]).unwrap();

    util::test_type(&conn,
                    "mood",
                    &[(Mood::Sad, "'sad'"), (Mood::Ok, "'ok'"), (Mood::Happy, "'happy'")]);
}

#[test]
fn wrong_name() {
    #[derive(Debug, ToSql, FromSql, PartialEq)]
    enum Foo {
        Bar,
        Baz
    }

    let conn = Connection::connect("postgres://postgres@localhost", TlsMode::None).unwrap();
    conn.execute("CREATE TYPE pg_temp.foo AS ENUM ('Bar', 'Baz')", &[]).unwrap();

    let err = conn.execute("SELECT $1::foo", &[&Foo::Bar]).unwrap_err();
    assert!(err.as_conversion().unwrap().is::<WrongType>());
}

#[test]
fn extra_variant() {
    #[derive(Debug, ToSql, FromSql, PartialEq)]
    #[postgres(name = "foo")]
    enum Foo {
        Bar,
        Baz,
        Buz,
    }

    let conn = Connection::connect("postgres://postgres@localhost", TlsMode::None).unwrap();
    conn.execute("CREATE TYPE pg_temp.foo AS ENUM ('Bar', 'Baz')", &[]).unwrap();

    let err = conn.execute("SELECT $1::foo", &[&Foo::Bar]).unwrap_err();
    assert!(err.as_conversion().unwrap().is::<WrongType>());
}

#[test]
fn missing_variant() {
    #[derive(Debug, ToSql, FromSql, PartialEq)]
    #[postgres(name = "foo")]
    enum Foo {
        Bar,
    }

    let conn = Connection::connect("postgres://postgres@localhost", TlsMode::None).unwrap();
    conn.execute("CREATE TYPE pg_temp.foo AS ENUM ('Bar', 'Baz')", &[]).unwrap();

    let err = conn.execute("SELECT $1::foo", &[&Foo::Bar]).unwrap_err();
    assert!(err.as_conversion().unwrap().is::<WrongType>());
}
