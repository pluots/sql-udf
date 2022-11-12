#![cfg(feature = "backend")]

mod backend;
use backend::get_db_connection;
use diesel::prelude::*;
use diesel::{sql_query, FromSqlRow};

const SETUP: &str = "
    create or replace table t1 (
        id int auto_increment,
        primary key (id)
    );

    insert into t1 (id) values (1);
    insert into t1 (id) values (2);
    insert into t1 (id) values (3);
    insert into t1 (id) values (4);
    insert into t1 (id) values (5);
    insert into t1 (id) values (6);

    ";

#[derive(QueryableByName, FromSqlRow, PartialEq, Debug)]
struct IntRes {
    #[diesel(sql_type = Option<i32>)]
    res: Option<i32>,
}

#[test]
fn test_single() {
    let conn = &mut get_db_connection();
    let q1 = sql_query("select udf_sequence() as res").load::<IntRes>(conn);
}
