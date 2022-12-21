#![cfg(feature = "backend")]

mod backend;

use backend::get_db_connection;
use diesel::dsl::sql;
use diesel::prelude::*;
use diesel::sql_types::Text;

const SETUP: [&str; 3] = [
    "create or replace function udf_attribute
        returns string
        soname 'libudf_examples.so'",
    "create or replace table test_attribute (
        id int auto_increment,
        val int,
        primary key (id)
    )",
    "insert into test_attribute (val) values (2)",
];

#[test]
fn test_basic() {
    let conn = &mut get_db_connection(&SETUP);

    let res: String =
        sql::<Text>("select udf_attribute(1, 'string', val, 3.2) from test_attribute")
            .get_result(conn)
            .expect("bad result");

    assert_eq!(res, "1, 'string', val, 3.2");
}
