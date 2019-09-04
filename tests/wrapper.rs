#[macro_use]
extern crate postgres_derive;
#[macro_use]
extern crate postgres;

use postgres::{Connection, TlsMode};
use postgres::types::WrongType;

mod util;

#[test]
fn transparent() {
    #[derive(FromSql, ToSql, Debug, PartialEq)]
    #[postgres(transparent)]
    struct ResourceId(i32);

    let conn = Connection::connect("postgres://postgres:password@localhost", TlsMode::None)
        .unwrap();

    util::test_type(
        &conn,
        "\"int4\"",
        &[
            (
                ResourceId(123),
                "123",
            ),
            (
                ResourceId(-27),
                "-27",
            ),
        ],
    );
}

#[test]
fn wrong_type() {
    #[derive(FromSql, ToSql, Debug, PartialEq)]
    #[postgres(transparent)]
    struct ResourceId(i32);

    let conn = Connection::connect("postgres://postgres:password@localhost", TlsMode::None)
        .unwrap();

    let err = conn.execute("SELECT $1::date", &[&ResourceId(0)])
        .unwrap_err();
    assert!(err.as_conversion().unwrap().is::<WrongType>());
}

