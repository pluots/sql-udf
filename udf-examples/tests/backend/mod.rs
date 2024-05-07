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
use std::sync::{Mutex, OnceLock};

use mysql::prelude::*;
use mysql::{Pool, PooledConn};

const URI_ENV: &str = "UDF_TEST_BACKEND_URI";
const DEFAULT_DATABASE_URI: &str = "mysql://root:example@0.0.0.0:12300/udf_tests";

static POOL: OnceLock<Pool> = OnceLock::new();
static SETUP_STATE: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();

fn get_database_uri() -> String {
    match env::var(URI_ENV) {
        Ok(s) => s,
        Err(_) => DEFAULT_DATABASE_URI.to_owned(),
    }
}

fn build_pool() -> Pool {
    let db_url = get_database_uri();

    {
        // Ensure the database exists then reconnect
        let (url, db) = db_url.rsplit_once('/').unwrap();
        let pool = Pool::new(url).expect("pool failed");
        let mut conn = pool.get_conn().expect("initial connection failed");

        // Create default database
        conn.query_drop(format!("CREATE OR REPLACE DATABASE {db}"))
            .unwrap();
    }

    Pool::new(db_url.as_str()).expect("pool failed")
}

/// Ensures that init items have been run
pub fn get_db_connection(init: &[&str]) -> PooledConn {
    let mut conn = POOL
        .get_or_init(build_pool)
        .get_conn()
        .expect("failed to get conn");

    let ran_stmts = &mut *SETUP_STATE
        .get_or_init(|| Mutex::new(HashSet::new()))
        .lock()
        .unwrap();

    // Store a list of our init calls so we don't repeat them
    for stmt in init {
        if ran_stmts.contains(*stmt) {
            continue;
        }

        conn.query_drop(stmt).expect("could not run setup");

        ran_stmts.insert((*stmt).to_owned());
    }

    conn
}

/// Check if two floats are within a tolerance. Also prints them for debugging.
#[allow(dead_code)]
pub fn approx_eq(a: f32, b: f32) -> bool {
    const TOLERANCE: f32 = 0.001;

    println!("a: {a}, b: {b}");
    (a - b).abs() < TOLERANCE
}
