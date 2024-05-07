#![cfg(feature = "backend")]

mod backend;

use backend::get_db_connection;
use mysql::prelude::*;

const SETUP: &[&str] = &[
    "CREATE OR REPLACE AGGREGATE FUNCTION udf_median
        RETURNS integer
        SONAME 'libudf_examples.so'",
    "CREATE OR REPLACE TABLE test_median (
        id int auto_increment,
        val int,
        primary key (id)
    )",
    "INSERT INTO test_median (val) VALUES (2), (1), (3), (4), (-3), (7), (-1)",
];

#[test]
fn test_empty() {
    let conn = &mut get_db_connection(SETUP);

    let res: i32 = conn
        .query_first("select udf_median(val) from test_median")
        .unwrap()
        .unwrap();

    assert_eq!(res, 2);
}
