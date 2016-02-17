#![feature(plugin, custom_derive)]
#![plugin(postgres_derive_macros)]

#[macro_use]
extern crate postgres;

use postgres::{Connection, SslMode};

mod util;

#[test]
fn defaults() {
    #[derive(ToSql, Debug)]
    struct InventoryItem {
        name: String,
        supplier_id: i32,
        price: f64,
    }

    let conn = Connection::connect("postgres://postgres@localhost", SslMode::None).unwrap();
    conn.batch_execute("CREATE TYPE pg_temp.\"InventoryItem\" AS (
                            name TEXT,
                            supplier_id INT,
                            price DOUBLE PRECISION
                        );

                        CREATE TEMPORARY TABLE foo (
                            item \"InventoryItem\"
                        );
                        ").unwrap();

    let item = InventoryItem {
        name: "foobar".to_owned(),
        supplier_id: 100,
        price: 15.50,
    };

    conn.execute("INSERT INTO foo (item) VALUES ($1)", &[&item]).unwrap();

    let rows = conn.query("SELECT (item).name, (item).supplier_id, (item).price
                           FROM foo", &[]).unwrap();
    let row = rows.get(0);
    assert_eq!(item.name, row.get::<_, String>(0));
    assert_eq!(item.supplier_id, row.get::<_, i32>(1));
    assert_eq!(item.price, row.get::<_, f64>(2));
}

#[test]
fn name_overrides() {
}
