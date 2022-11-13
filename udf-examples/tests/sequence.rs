#![cfg(feature = "backend")]

mod backend;

use backend::get_db_connection;
use diesel::dsl::sql;
use diesel::prelude::*;
use diesel::sql_types::Integer;

const SETUP: [&str; 3] = [
    "create or replace function udf_sequence
        returns integer
        soname 'libudf_examples.so'",
    "create or replace table test_seq (
        id int
    )",
    "insert into test_seq (id) values (1), (2), (3), (4), (5), (6)",
];

#[test]
fn test_single() {
    let conn = &mut get_db_connection(&SETUP);

    // First result should be 1
    let res: (i32,) = sql::<(Integer,)>("select udf_sequence()")
        .get_result(conn)
        .expect("bad result");

    assert_eq!(res.0, 1);
}

#[test]
fn test_offset() {
    let conn = &mut get_db_connection(&SETUP);

    // With argument specified, we should have one more than the
    // specified value
    let res: (i32,) = sql::<(Integer,)>("select udf_sequence(4)")
        .get_result(conn)
        .expect("bad result");

    assert_eq!(res.0, 5);
}

#[test]
fn test_incrementing() {
    let conn = &mut get_db_connection(&SETUP);

    // Test results with multiple rows
    let res: Vec<(i32, i32)> = sql::<(Integer, Integer)>("select id, udf_sequence() from test_seq")
        .get_results(conn)
        .expect("bad result");

    assert_eq!(res, vec![(1, 1), (2, 2), (3, 3), (4, 4), (5, 5), (6, 6)]);
}

#[test]
fn test_incrementing_offset() {
    let conn = &mut get_db_connection(&SETUP);

    // Test results with multiple rows
    let res: Vec<(i32, i32)> =
        sql::<(Integer, Integer)>("select id, udf_sequence(10) from test_seq")
            .get_results(conn)
            .expect("bad result");

    assert_eq!(
        res,
        vec![(1, 11), (2, 12), (3, 13), (4, 14), (5, 15), (6, 16)]
    );
}

#[test]
fn test_incrementing_offset_negative() {
    let conn = &mut get_db_connection(&SETUP);

    // Test results with multiple rows
    let res: Vec<(i32, i32)> =
        sql::<(Integer, Integer)>("select id, udf_sequence(-2) from test_seq")
            .get_results(conn)
            .expect("bad result");

    assert_eq!(res, vec![(1, -1), (2, 0), (3, 1), (4, 2), (5, 3), (6, 4)]);
}
