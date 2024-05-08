#![cfg(feature = "backend")]

mod backend;

use backend::get_db_connection;
use mysql::prelude::*;

const SETUP: &[&str] = &[
    "CREATE OR REPLACE FUNCTION udf_sequence
        RETURNS integer
        SONAME 'libudf_examples.so'",
    "CREATE OR REPLACE TABLE test_seq (
        id int
    )",
    "INSERT INTO test_seq (id) VALUES (1), (2), (3), (4), (5), (6)",
];

#[test]
fn test_single() {
    let conn = &mut get_db_connection(SETUP);

    // First result should be 1
    let res: i32 = conn.query_first("select udf_sequence()").unwrap().unwrap();

    assert_eq!(res, 1);
}

#[test]
fn test_offset() {
    let conn = &mut get_db_connection(SETUP);

    // With argument specified, we should have one more than the
    // specified value
    let res: i32 = conn.query_first("select udf_sequence(4)").unwrap().unwrap();

    assert_eq!(res, 5);
}

#[test]
fn test_incrementing() {
    let conn = &mut get_db_connection(SETUP);

    // Test results with multiple rows
    let res: Vec<(i32, i32)> = conn
        .query("select id, udf_sequence() from test_seq")
        .unwrap();

    assert_eq!(res, vec![(1, 1), (2, 2), (3, 3), (4, 4), (5, 5), (6, 6)]);
}

#[test]
fn test_incrementing_offset() {
    let conn = &mut get_db_connection(SETUP);

    // Test results with multiple rows
    let res: Vec<(i32, i32)> = conn
        .query("select id, udf_sequence(10) from test_seq")
        .unwrap();

    assert_eq!(
        res,
        vec![(1, 11), (2, 12), (3, 13), (4, 14), (5, 15), (6, 16)]
    );
}

#[test]
fn test_incrementing_offset_negative() {
    let conn = &mut get_db_connection(SETUP);

    // Test results with multiple rows
    let res: Vec<(i32, i32)> = conn
        .query("select id, udf_sequence(-2) from test_seq")
        .unwrap();

    assert_eq!(res, vec![(1, -1), (2, 0), (3, 1), (4, 2), (5, 3), (6, 4)]);
}
