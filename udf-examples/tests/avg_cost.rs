#![cfg(feature = "backend")]

mod backend;

use backend::{approx_eq, get_db_connection};
use mysql::prelude::*;

const SETUP: &[&str] = &[
    "CREATE OR REPLACE AGGREGATE FUNCTION avg_cost
        RETURNS real
        SONAME 'libudf_examples.so'",
    "CREATE OR REPLACE TABLE test_avgcost (
        id int auto_increment,
        qty int,
        cost real,
        class varchar(30),
        primary key (id)
    )",
    r#"INSERT INTO test_avgcost (qty, cost, class) values
        (10, 50, "a"),
        (8, 5.6, "c"),
        (5, 20.7, "a"),
        (10, 12.78, "b"),
        (6, 7.2, "c"),
        (2, 10.3, "b"),
        (3, 9.1, "c")
    "#,
];

#[test]
fn test_empty() {
    let conn = &mut get_db_connection(SETUP);

    let res: Vec<f32> = conn
        .query("SELECT avg_cost(qty, cost) FROM test_avgcost GROUP BY class")
        .unwrap();

    println!("{res:?}");
    assert_eq!(res.len(), 3);

    let expected = [40.233, 12.366, 6.782];
    for (a, b) in res.iter().zip(expected.iter()) {
        assert!(approx_eq(*a, *b));
    }
}
