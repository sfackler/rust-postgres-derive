#[macro_use]
extern crate postgres_derive;
#[macro_use]
extern crate postgres;

use postgres::{Connection, TlsMode};
use postgres::types::WrongType;

mod util;

#[test]
fn defaults() {
    #[derive(FromSql, ToSql, Debug, PartialEq)]
    #[postgres(schema = "pg_temp")]
    struct SessionId(Vec<u8>);

    let conn = Connection::connect("postgres://postgres:password@localhost", TlsMode::None)
        .unwrap();
    conn.execute(
        "CREATE DOMAIN pg_temp.\"SessionId\" AS bytea CHECK(octet_length(VALUE) = 16);",
        &[],
    ).unwrap();

    util::test_type(
        &conn,
        "\"SessionId\"",
        &[
            (
                SessionId(b"0123456789abcdef".to_vec()),
                "'0123456789abcdef'",
            ),
        ],
    );
}

#[test]
fn name_overrides() {
    #[derive(FromSql, ToSql, Debug, PartialEq)]
    #[postgres(schema = "pg_temp", name = "session_id")]
    struct SessionId(Vec<u8>);

    let conn = Connection::connect("postgres://postgres:password@localhost", TlsMode::None)
        .unwrap();
    conn.execute(
        "CREATE DOMAIN pg_temp.session_id AS bytea CHECK(octet_length(VALUE) = 16);",
        &[],
    ).unwrap();

    util::test_type(
        &conn,
        "session_id",
        &[
            (
                SessionId(b"0123456789abcdef".to_vec()),
                "'0123456789abcdef'",
            ),
        ],
    );
}

#[test]
fn wrong_name() {
    #[derive(FromSql, ToSql, Debug, PartialEq)]
    #[postgres(schema = "pg_temp")]
    struct SessionId(Vec<u8>);

    let conn = Connection::connect("postgres://postgres:password@localhost", TlsMode::None)
        .unwrap();
    conn.execute(
        "CREATE DOMAIN pg_temp.session_id AS bytea CHECK(octet_length(VALUE) = 16);",
        &[],
    ).unwrap();

    let err = conn.execute("SELECT $1::session_id", &[&SessionId(vec![])])
        .unwrap_err();
    assert!(err.as_conversion().unwrap().is::<WrongType>());
}

#[test]
fn wrong_type() {
    #[derive(FromSql, ToSql, Debug, PartialEq)]
    #[postgres(schema = "pg_temp", name = "session_id")]
    struct SessionId(i32);

    let conn = Connection::connect("postgres://postgres:password@localhost", TlsMode::None)
        .unwrap();
    conn.execute(
        "CREATE DOMAIN pg_temp.session_id AS bytea CHECK(octet_length(VALUE) = 16);",
        &[],
    ).unwrap();

    let err = conn.execute("SELECT $1::session_id", &[&SessionId(0)])
        .unwrap_err();
    assert!(err.as_conversion().unwrap().is::<WrongType>());
}
