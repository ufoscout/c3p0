#![cfg(feature = "sqlite")]

use r2d2_sqlite::SqliteConnectionManager;
use c3p0::prelude::*;
use testcontainers::*;

pub fn new_connection(
    _docker: &clients::Cli,
) -> (C3p0, String) {

    let manager = SqliteConnectionManager::memory();

    let pool = r2d2::Pool::builder().build(manager).unwrap();

    let pool = C3p0Builder::build(pool);

    (pool, "".to_owned())
}
