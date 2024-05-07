#![cfg(feature = "backend")]

mod backend;

use backend::get_db_connection;
use mysql::prelude::*;

const SETUP: &[&str] = &["create or replace function lookup6
        returns string
        soname 'libudf_examples.so'"];

#[test]
fn test_zeros() {
    let conn = &mut get_db_connection(SETUP);

    let res: Option<String> = conn
        .query_first("select lookup6('0.0.0.0')")
        .unwrap()
        .unwrap();

    assert_eq!(res.unwrap(), "::ffff:0.0.0.0");
}

#[test]
fn test_localhost() {
    let conn = &mut get_db_connection(SETUP);

    let res: Option<String> = conn
        .query_first("select lookup6('localhost')")
        .unwrap()
        .unwrap();

    assert_eq!(res.unwrap(), "::1");
}

#[test]
fn test_nonexistant() {
    let conn = &mut get_db_connection(SETUP);

    let res: Option<String> = conn
        .query_first("select lookup6('nonexistant')")
        .unwrap()
        .unwrap();

    assert!(res.is_none());
}

#[test]
fn test_sql_buffer_bug() {
    // This is intended to catch a buffer problem in mysql/mariadb
    // See link: https://github.com/pluots/sql-udf/issues/39

    let conn = &mut get_db_connection(SETUP);

    conn.exec_drop("set @testval = (select lookup6('0.0.0.0'))", ())
        .unwrap();

    let res: Option<String> = conn
        .query_first("select  regexp_replace(@testval,'[:.]','')")
        .unwrap()
        .unwrap();

    assert_eq!(res.unwrap(), "ffff0000");
}
