#![feature(plugin, custom_derive)]
#![plugin(postgres_derive_macros)]

#[macro_use]
extern crate postgres;

use postgres::types::{FromSql, ToSql};
use postgres::{Connection, SslMode};
use std::fmt;

fn test_type<T, S>(conn: &Connection, sql_type: &str, checks: &[(T, S)])
    where T: PartialEq + FromSql + ToSql, S: fmt::Display
{
    for &(ref val, ref repr) in checks.iter() {
        let stmt = conn.prepare(&*format!("SELECT {}::{}", *repr, sql_type)).unwrap();
        let result = stmt.query(&[]).unwrap().iter().next().unwrap().get(0);
        assert_eq!(val, &result);

        let stmt = conn.prepare(&*format!("SELECT $1::{}", sql_type)).unwrap();
        let result = stmt.query(&[val]).unwrap().iter().next().unwrap().get(0);
        assert_eq!(val, &result);
    }
}

#[test]
fn defaults() {
    #[derive(Debug, ToSql, FromSql, PartialEq)]
    enum Foo {
        Bar,
        Baz
    }

    let conn = Connection::connect("postgres://postgres@localhost", SslMode::None).unwrap();
    conn.execute("CREATE TYPE pg_temp.\"Foo\" AS ENUM ('Bar', 'Baz')", &[]).unwrap();

    test_type(&conn, "\"Foo\"", &[(Foo::Bar, "'Bar'"), (Foo::Baz, "'Baz'")]);
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

    let conn = Connection::connect("postgres://postgres@localhost", SslMode::None).unwrap();
    conn.execute("CREATE TYPE pg_temp.mood AS ENUM ('sad', 'ok', 'happy')", &[]).unwrap();

    test_type(&conn, "mood", &[(Mood::Sad, "'sad'"), (Mood::Ok, "'ok'"), (Mood::Happy, "'happy'")]);
}
