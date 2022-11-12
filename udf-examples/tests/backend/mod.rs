//! Module to run tests when a backend is available
//!
//! This module requires docker to be available on the host system, with the
//! image called `mdb-example-so`. If that is available, run these tests with
//! `cargo t --features backend`

#![cfg(feature = "backend")]

use std::ffi::{OsStr, OsString};
use std::os::unix::ffi::OsStrExt;
use std::process::Command;
use std::sync::Mutex;

use diesel::mysql::MysqlConnection;
use diesel::Connection;

const NAME: &str = "mdb-example-so";
const DATABASE_URL: &str = "mysql://root:example@0.0.0.0:12300/db";

// Uninitialized by default
static DOCKER_CONTAINER: Mutex<Option<MariaDbContainer>> = Mutex::new(None);

#[derive(Debug)]
struct MariaDbContainer {
    hash: OsString,
}

impl MariaDbContainer {
    fn initialize() -> Self {
        let hash_bytes = Command::new("docker")
            .args(["run", "--rm", "-it", "-d", "-p", "12300:3306", NAME])
            .output()
            .expect("failed to start docker process")
            .stdout;

        let hash = OsStr::from_bytes(&hash_bytes).to_os_string();
        eprintln!("started {:?}", hash);
        Self { hash }
    }
}

impl Drop for MariaDbContainer {
    fn drop(&mut self) {
        Command::new("docker").arg("stop").arg(&self.hash);
    }
}

fn make_connection() -> MysqlConnection {
    MysqlConnection::establish(DATABASE_URL)
        .unwrap_or_else(|e| panic!("Error connecting to {}: {e}", DATABASE_URL))
}

/// Start the container if it hasn't been yet
pub fn get_db_connection() -> MysqlConnection {
    let mut guard = DOCKER_CONTAINER.lock().unwrap();
    // If the container hasn't yet been initialized, do so
    if guard.is_none() {
        eprintln!("INITING");
        *guard = Some(MariaDbContainer::initialize());
    }
    eprintln!("{guard:?}");
    // sleep(std::time::Duration::from_secs(10));

    make_connection()
}

#[test]
fn x() {
    get_db_connection();
}
