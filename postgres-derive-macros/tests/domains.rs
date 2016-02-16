#![feature(plugin, custom_derive)]
#![plugin(postgres_derive_macros)]

#[macro_use]
extern crate postgres;

use postgres::{Connection, SslMode};

mod util;

#[test]
fn defaults() {
    #[derive(FromSql, ToSql, Debug, PartialEq)]
    struct SessionId(Vec<u8>);

    let conn = Connection::connect("postgres://postgres@localhost", SslMode::None).unwrap();
    conn.execute("CREATE DOMAIN pg_temp.\"SessionId\" AS bytea CHECK(octet_length(VALUE) = 16);",
                 &[])
        .unwrap();

    util::test_type(&conn, "\"SessionId\"", &[(SessionId(b"0123456789abcdef".to_vec()),
                                               "'0123456789abcdef'")]);
}

#[test]
fn name_overrides() {
    #[derive(FromSql, ToSql, Debug, PartialEq)]
    #[postgres(name = "session_id")]
    struct SessionId(Vec<u8>);

    let conn = Connection::connect("postgres://postgres@localhost", SslMode::None).unwrap();
    conn.execute("CREATE DOMAIN pg_temp.session_id AS bytea CHECK(octet_length(VALUE) = 16);", &[])
        .unwrap();

    util::test_type(&conn, "session_id", &[(SessionId(b"0123456789abcdef".to_vec()),
                                            "'0123456789abcdef'")]);
}
