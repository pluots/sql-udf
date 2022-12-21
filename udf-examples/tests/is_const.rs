#![cfg(feature = "backend")]

mod backend;

use backend::get_db_connection;
use diesel::dsl::sql;
use diesel::prelude::*;
use diesel::result::Error as DieselError;
use diesel::sql_types::Text;

const SETUP: [&str; 3] = [
    "create or replace function is_const
        returns string
        soname 'libudf_examples.so'",
    "create or replace table test_is_const (
        id int auto_increment,
        val int,
        primary key (id)
    )",
    "insert into test_is_const (val) values (2)",
];

#[test]
fn test_true() {
    let conn = &mut get_db_connection(&SETUP);

    let res: String = sql::<Text>("select is_const(1)")
        .get_result(conn)
        .expect("bad result");

    assert_eq!(res, "const");
}

#[test]
fn test_false() {
    let conn = &mut get_db_connection(&SETUP);

    let res: String = sql::<Text>("select is_const(val) from test_is_const")
        .get_result(conn)
        .expect("bad result");

    assert_eq!(res, "not const");
}

#[test]
fn test_too_many_args() {
    let conn = &mut get_db_connection(&SETUP);

    let res = sql::<Text>("select is_const(1, 2)").get_result::<String>(conn);

    let Err(DieselError::DatabaseError(_, info)) = res else {
        panic!("Got unexpected response: {res:?}");
    };

    assert!(info.message().contains("only accepts one argument"));
}
