#![cfg(feature = "backend")]

mod backend;

use backend::get_db_connection;
use mysql::prelude::*;

const SETUP: &[&str] = &["create or replace function mishmash
        returns string
        soname 'libudf_examples.so'"];

#[test]
fn test_empty() {
    let conn = &mut get_db_connection(SETUP);

    let res: Option<Vec<u8>> = conn.query_first("select mishmash()").unwrap().unwrap();

    assert!(res.is_none());
}

#[test]
fn test_single() {
    let conn = &mut get_db_connection(SETUP);

    let res: Option<Vec<u8>> = conn
        .query_first("select mishmash('banana')")
        .unwrap()
        .unwrap();

    assert_eq!(res.unwrap(), b"banana");
}

#[test]
fn test_many() {
    let conn = &mut get_db_connection(SETUP);

    let res: Option<Vec<u8>> = conn
        .query_first("select mishmash('banana', 'is', 'a', 'fruit')")
        .unwrap()
        .unwrap();

    assert_eq!(res.unwrap(), b"bananaisafruit");
}
