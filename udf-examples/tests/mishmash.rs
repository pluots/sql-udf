#![cfg(feature = "backend")]

mod backend;

use backend::get_db_connection;
use diesel::dsl::sql;
use diesel::prelude::*;
use diesel::sql_types::{Binary, Nullable};

const SETUP: [&str; 1] = ["create or replace function mishmash
        returns string
        soname 'libudf_examples.so'"];

#[test]
fn test_empty() {
    let conn = &mut get_db_connection(&SETUP);

    let res: Option<Vec<u8>> = sql::<Nullable<Binary>>("select mishmash()")
        .get_result(conn)
        .expect("bad result");

    assert!(res.is_none());
}

#[test]
fn test_single() {
    let conn = &mut get_db_connection(&SETUP);

    let res: Option<Vec<u8>> = sql::<Nullable<Binary>>("select mishmash('banana')")
        .get_result(conn)
        .expect("bad result");

    assert_eq!(res.unwrap(), b"banana");
}

#[test]
fn test_many() {
    let conn = &mut get_db_connection(&SETUP);

    let res: Option<Vec<u8>> =
        sql::<Nullable<Binary>>("select mishmash('banana', 'is', 'a', 'fruit')")
            .get_result(conn)
            .expect("bad result");

    assert_eq!(res.unwrap(), b"bananaisafruit");
}
