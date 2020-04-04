#![cfg(feature = "sqlite_blocking")]

use c3p0_sqlite::sqlite::r2d2::{Pool, SqliteConnectionManager};
use c3p0_sqlite::sqlite::*;
use c3p0_sqlite::*;
use testcontainers::*;

mod tests_blocking;
pub mod utils;

pub fn new_connection(_docker: &clients::Cli) -> (SqliteC3p0Pool, String) {
    let manager = SqliteConnectionManager::memory();

    let pool = Pool::builder().build(manager).unwrap();

    let pool = SqliteC3p0Pool::new(pool);

    (pool, "".to_owned())
}
