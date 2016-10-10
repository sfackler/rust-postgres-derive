use postgres::types::{FromSql, ToSql};
use postgres::Connection;
use std::fmt;

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
