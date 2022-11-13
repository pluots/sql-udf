#![cfg(feature = "backend")]

mod backend;

use backend::get_db_connection;
use diesel::dsl::sql;
use diesel::prelude::*;
use diesel::sql_types::Integer;

const SETUP: [&str; 3] = [
    "create or replace aggregate function udf_median
        returns integer
        soname 'libudf_examples.so'",
    "create or replace table test_median (
        id int auto_increment,
        val int,
        primary key (id)
    )",
    "insert into test_median (val) values (2), (1), (3), (4), (-3), (7), (-1)",
];

#[test]
fn test_empty() {
    let conn = &mut get_db_connection(&SETUP);

    let res: (i32,) = sql::<(Integer,)>("select udf_median(val) from test_median")
        .get_result(conn)
        .expect("bad result");

    assert_eq!(res.0, 2);
}
