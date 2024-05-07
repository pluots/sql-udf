#![cfg(feature = "backend")]

mod backend;

use backend::get_db_connection;
use mysql::prelude::*;

const SETUP: &[&str] = &["create or replace function sum_int
        returns integer
        soname 'libudf_examples.so'"];

#[test]
fn test_empty() {
    let conn = &mut get_db_connection(SETUP);

    let res: i32 = conn.query_first("select sum_int()").unwrap().unwrap();

    assert_eq!(res, 0);
}

#[test]
fn test_basic() {
    let conn = &mut get_db_connection(SETUP);

    let res: i32 = conn
        .query_first("select sum_int(1, 2, 3, 4, -6)")
        .unwrap()
        .unwrap();

    assert_eq!(res, 4);
}

#[test]
fn test_coercion() {
    let conn = &mut get_db_connection(SETUP);

    let res: i32 = conn
        .query_first("select sum_int(1, 2, '-5', '11')")
        .unwrap()
        .unwrap();

    assert_eq!(res, 9);
}
