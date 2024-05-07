#![cfg(feature = "backend")]

mod backend;

use backend::{approx_eq, get_db_connection};
use mysql::prelude::*;

const SETUP: &[&str] = &[
    "CREATE OR REPLACE AGGREGATE function avg2
        RETURNS real
        SONAME 'libudf_examples.so'",
    "CREATE OR REPLACE AGGREGATE FUNCTION test_avg2_alias
        RETURNS real
        SONAME 'libudf_examples.so'",
    "CREATE OR REPLACE TABLE test_avg2 (
        id int auto_increment,
        val int,
        primary key (id)
    )",
    "INSERT INTO test_avg2 (val) VALUES (2), (1), (3), (4), (-3), (7), (-1)",
];

#[test]
fn test_avg2() {
    let conn = &mut get_db_connection(SETUP);

    let res: f32 = conn
        .query_first("select avg2(val) from test_avg2")
        .unwrap()
        .unwrap();

    assert!(approx_eq(res, 1.857));
}

#[test]
fn test_avg2_alias() {
    let conn = &mut get_db_connection(SETUP);

    let res: f32 = conn
        .query_first("select test_avg2_alias(val) from test_avg2")
        .unwrap()
        .unwrap();

    assert!(approx_eq(res, 1.857));
}
