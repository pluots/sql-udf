#![cfg(feature = "backend")]

mod backend;

use backend::get_db_connection;
use diesel::dsl::sql;
use diesel::prelude::*;
use diesel::sql_types::{Nullable, Text};

const SETUP: [&str; 1] = ["create or replace function lipsum
        returns string
        soname 'libudf_examples.so'"];

#[test]
fn test_short() {
    let conn = &mut get_db_connection(&SETUP);

    let res: Option<String> = sql::<Nullable<Text>>("select lipsum(10)")
        .get_result(conn)
        .expect("bad result");

    assert!(res.unwrap().split_whitespace().count() == 10);
}

#[test]
fn test_short_seed() {
    let conn = &mut get_db_connection(&SETUP);

    let res: Option<String> = sql::<Nullable<Text>>("select lipsum(10, 12345)")
        .get_result(conn)
        .expect("bad result");

    assert!(res.unwrap().split_whitespace().count() == 10);
}

#[test]
fn test_long() {
    let conn = &mut get_db_connection(&SETUP);

    let res: Option<String> = sql::<Nullable<Text>>("select lipsum(5000)")
        .get_result(conn)
        .expect("bad result");

    assert!(res.unwrap().split_whitespace().count() == 5000);
}
