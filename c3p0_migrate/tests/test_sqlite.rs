#![cfg(feature = "sqlite")]

use c3p0_pool_sqlite::r2d2::{Pool, SqliteConnectionManager};
use testcontainers::*;
use c3p0_pool_sqlite::SqlitePoolManager;
use c3p0_common::C3p0Pool;

mod tests;

pub fn new_connection(_docker: &clients::Cli) -> (C3p0Pool<SqlitePoolManager>, String) {
    let manager = SqliteConnectionManager::memory();

    let pool = Pool::builder().build(manager).unwrap();

    let pool = C3p0Pool::new(SqlitePoolManager::new(pool));

    (pool, "".to_owned())
}
