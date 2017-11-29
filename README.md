# postgres-derive

[![CircleCI](https://circleci.com/gh/sfackler/rust-postgres-derive.svg?style=shield)](https://circleci.com/gh/sfackler/rust-postgres-derive)

Syntax extensions to automatically derive `FromSql` and `ToSql` implementations for Postgres enum,
domain, and composite types.

The generated code requires rust-postgres 0.14 or 0.15 and Rust 1.15.0 or higher.

# Usage

Simply depend on the `postgres-derive` crate and register it as a plugin:

Cargo.toml
```toml
# ...

[dependencies]
postgres-derive = "0.3"
postgres = "0.15"
```

lib.rs
```rust
#[macro_use]
extern crate postgres;
#[macro_use]
extern crate postgres_derive;

#[derive(Debug, ToSql, FromSql)]
pub enum Mood {
    Sad,
    Ok,
    Happy,
}

// ...
```

# Types

## Enums

Postgres enums correspond to C-like enums in Rust:

```sql
CREATE TYPE "Mood" AS ENUM (
    'Sad',
    'Ok',
    'Happy'
);
```

```rust
#[derive(Debug, ToSql, FromSql)]
enum Mood {
    Sad,
    Ok,
    Happy,
}
```

The implementations will expect exact matches between the type names and variants. The
`#[postgres(...)]` attribute can be used to adjust the names used on the Postgres side:

```sql
CREATE TYPE people.mood AS ENUM (
    'sad',
    'ok',
    'happy'
);
```

```rust
#[derive(Debug, ToSql, FromSql)]
#[postgres(schema = "people", name = "mood")]
enum Mood {
    #[postgres(name = "sad")]
    Sad,
    #[postgres(name = "ok")]
    Ok,
    #[postgres(name = "happy")]
    Happy,
}
```

## Domains

Postgres domains correspond to tuple structs with one member in Rust:

```sql
CREATE DOMAIN "SessionId" AS BYTEA CHECK(octet_length(VALUE) = 16);
```

```rust
#[derive(Debug, FromSql, ToSql)]
struct SessionId(Vec<u8>);
```

As above, the implementations will expect an exact match between the Rust and Postgres type names,
and the `#[postgres(...)]` attribute can be used to adjust that behavior:

```sql
CREATE DOMAIN accounts.session_id AS BYTEA CHECK(octet_length(VALUE) = 16);
```

```rust
#[derive(Debug, FromSql, ToSql)]
#[postgres(schema = "accounts", name = "session_id")]
struct SessionId(Vec<u8>);
```

## Composites

Postgres composite types correspond to structs in Rust:

```sql
CREATE TYPE "InventoryItem" AS (
    name TEXT,
    supplier_id INT,
    price DOUBLE PRECISION
);
```

```rust
#[derive(Debug, FromSql, ToSql)]
struct InventoryItem {
    name: String,
    supplier_id: i32,
    price: Option<f64>,
}
```

Again, the implementations will expect an exact match between the names of the Rust and Postgres
types and fields, which can be adjusted via the `#[postgres(...)]` attribute:


```sql
CREATE TYPE shop.inventory_item AS (
    name TEXT,
    supplier_id INT,
    the_price DOUBLE PRECISION
);
```

```rust
#[derive(Debug, FromSql, ToSql)]
#[postgres(schema = "shop", name = "inventory_item")]
struct InventoryItem {
    name: String,
    supplier_id: i32,
    #[postgres(name = "the_price")]
    price: Option<f64>,
}
```

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
