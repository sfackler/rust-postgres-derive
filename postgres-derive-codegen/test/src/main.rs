#[macro_use]
extern crate postgres;

use postgres::{Connection, SslMode};
use postgres::types::{FromSql, ToSql};
use std::fmt;

include!(concat!(env!("OUT_DIR"), "/types.rs"));

pub fn test_type<T, S>(conn: &Connection, sql_type: &str, checks: &[(T, S)])
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
fn domain() {
    let conn = Connection::connect("postgres://postgres@localhost", SslMode::None).unwrap();
    conn.execute("CREATE DOMAIN pg_temp.session_id AS bytea CHECK(octet_length(VALUE) = 16);", &[])
        .unwrap();

    test_type(&conn, "session_id", &[(SessionId(b"0123456789abcdef".to_vec()),
                                      "'0123456789abcdef'")]);
}

#[test]
fn enum_() {
    let conn = Connection::connect("postgres://postgres@localhost", SslMode::None).unwrap();
    conn.execute("CREATE TYPE pg_temp.mood AS ENUM ('sad', 'ok', 'happy')", &[]).unwrap();

    test_type(&conn,
              "mood",
              &[(Mood::Sad, "'sad'"), (Mood::Ok, "'ok'"), (Mood::Happy, "'happy'")]);
}
