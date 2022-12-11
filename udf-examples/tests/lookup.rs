#![cfg(feature = "backend")]

mod backend;

use backend::get_db_connection;
use diesel::dsl::sql;
use diesel::prelude::*;
use diesel::sql_types::{Nullable, Text, Untyped};

const SETUP: [&str; 1] = ["create or replace function lookup6
        returns string
        soname 'libudf_examples.so'"];

#[test]
fn test_zeros() {
    let conn = &mut get_db_connection(&SETUP);

    let res: (Option<String>,) = sql::<(Nullable<Text>,)>("select lookup6('0.0.0.0')")
        .get_result(conn)
        .expect("bad result");

    assert_eq!(res.0.unwrap(), "::ffff:0.0.0.0");
}

#[test]
fn test_localhost() {
    let conn = &mut get_db_connection(&SETUP);

    let res: (Option<String>,) = sql::<(Nullable<Text>,)>("select lookup6('localhost')")
        .get_result(conn)
        .expect("bad result");

    assert_eq!(res.0.unwrap(), "::1");
}

#[test]
fn test_nonexistant() {
    let conn = &mut get_db_connection(&SETUP);

    let res: (Option<String>,) = sql::<(Nullable<Text>,)>("select lookup6('nonexistant')")
        .get_result(conn)
        .expect("bad result");

    assert!(res.0.is_none());
}

#[test]
fn test_sql_buffer_bug() {
    // This is intended to catch a buffer problem in mysql/mariadb
    // See link: https://github.com/pluots/sql-udf/issues/39

    let conn = &mut get_db_connection(&SETUP);

    sql::<(Untyped,)>("set @testval = (select lookup6('0.0.0.0'))")
        .execute(conn)
        .unwrap();

    let res: (Option<String>,) =
        sql::<(Nullable<Text>,)>("select  regexp_replace(@testval,'[:.]','')")
            .get_result(conn)
            .expect("bad result");

    assert_eq!(res.0.unwrap(), "ffff0000");
}
