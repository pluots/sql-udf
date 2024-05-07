#![cfg(feature = "backend")]

mod backend;

use backend::get_db_connection;
use mysql::prelude::*;

const SETUP: &[&str] = &["create or replace function lipsum
        returns string
        soname 'libudf_examples.so'"];

#[test]
fn test_short() {
    let conn = &mut get_db_connection(SETUP);

    let res: Option<String> = conn.query_first("select lipsum(10)").unwrap().unwrap();

    assert!(res.unwrap().split_whitespace().count() == 10);
}

#[test]
fn test_short_seed() {
    let conn = &mut get_db_connection(SETUP);

    let res: Option<String> = conn
        .query_first("select lipsum(10, 12345)")
        .unwrap()
        .unwrap();

    assert!(res.unwrap().split_whitespace().count() == 10);
}

#[test]
fn test_long() {
    let conn = &mut get_db_connection(SETUP);

    let res: Option<String> = conn.query_first("select lipsum(5000)").unwrap().unwrap();
    assert!(res.unwrap().split_whitespace().count() == 5000);
}
