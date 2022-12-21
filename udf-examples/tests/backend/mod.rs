//! Module to run tests when a backend is available
//!
//! This module requires a database that matches the description in
//! `DEFAULT_DATABASE_URI`. If that is available, run these tests with `cargo t
//! --features backend`
//!
//! Run the container with `docker run --rm -d -p 12300:3300 mdb-example-so`

#![cfg(feature = "backend")]
use std::collections::HashSet;
use std::env;
use std::sync::{Mutex, Once};

use diesel::dsl::sql;
use diesel::mysql::MysqlConnection;
use diesel::prelude::*;
use diesel::sql_types::Untyped;
use diesel::Connection;
use lazy_static::lazy_static;

const URI_ENV: &str = "UDF_TEST_BACKEND_URI";
const DEFAULT_DATABASE_URI: &str = "mysql://root:example@0.0.0.0:12300/udf_tests";

static INIT: Once = Once::new();

lazy_static! {
    static ref SETUP_STATE: Mutex<HashSet<String>> = Mutex::new(HashSet::new());
}

fn get_database_uri() -> String {
    match env::var(URI_ENV) {
        Ok(s) => s,
        Err(_) => DEFAULT_DATABASE_URI.to_owned(),
    }
}

/// Ensure the init items have been run
pub fn get_db_connection(init: &[&str]) -> MysqlConnection {
    let db_url = get_database_uri();

    INIT.call_once(|| {
        let mut conn = MysqlConnection::establish(db_url.rsplit_once('/').unwrap().0)
            .expect("initial connection failed");

        sql::<Untyped>("create or replace database udf_tests")
            .execute(&mut conn)
            .expect("could not create databases");
    });

    let hset = &mut *SETUP_STATE.lock().unwrap();
    let mut conn = MysqlConnection::establish(&db_url).expect("initial connection failed");

    // Store a list of our init calls so we don't repeat them
    for stmt in init {
        if hset.contains(*stmt) {
            continue;
        }
        sql::<Untyped>(stmt)
            .execute(&mut conn)
            .expect("could not run setup");

        hset.insert((*stmt).to_owned());
    }

    conn
}
