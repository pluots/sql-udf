#![cfg(feature = "backend")]

mod backend;

use backend::get_db_connection;
use mysql::prelude::*;

const SETUP: &[&str] = &[
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
    let conn = &mut get_db_connection(SETUP);

    let res: String = conn.query_first("select is_const(1)").unwrap().unwrap();

    assert_eq!(res, "const");
}

#[test]
fn test_false() {
    let conn = &mut get_db_connection(SETUP);

    let res: String = conn
        .query_first("select is_const(val) from test_is_const")
        .unwrap()
        .unwrap();

    assert_eq!(res, "not const");
}

#[test]
fn test_too_many_args() {
    let conn = &mut get_db_connection(SETUP);

    let res = conn.query_first::<String, _>("select is_const(1, 2)");

    let Err(mysql::Error::MySqlError(e)) = res else {
        panic!("Got unexpected response: {res:?}");
    };

    assert!(e.message.contains("only accepts one argument"));
}
