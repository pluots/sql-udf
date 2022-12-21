#![cfg(feature = "backend")]

mod backend;

use backend::get_db_connection;
use diesel::dsl::sql;
use diesel::prelude::*;
use diesel::sql_types::Integer;

const SETUP: [&str; 1] = ["create or replace function sum_int
        returns integer
        soname 'libudf_examples.so'"];

#[test]
fn test_empty() {
    let conn = &mut get_db_connection(&SETUP);

    let res: i32 = sql::<Integer>("select sum_int()")
        .get_result(conn)
        .expect("bad result");

    assert_eq!(res, 0);
}

#[test]
fn test_basic() {
    let conn = &mut get_db_connection(&SETUP);

    let res: i32 = sql::<Integer>("select sum_int(1, 2, 3, 4, -6)")
        .get_result(conn)
        .expect("bad result");

    assert_eq!(res, 4);
}

#[test]
fn test_coercion() {
    let conn = &mut get_db_connection(&SETUP);

    let res: i32 = sql::<Integer>("select sum_int(1, 2, '-5', '11')")
        .get_result(conn)
        .expect("bad result");

    assert_eq!(res, 9);
}
