#![cfg(feature = "sqlite")]

use c3p0_json::*;
use c3p0_json::sqlite::r2d2::SqliteConnectionManager;
use testcontainers::*;

pub use c3p0_json::sqlite::C3p0Sqlite as C3p0;
pub use c3p0_json::sqlite::C3p0SqliteBuilder as C3p0Builder;

pub fn new_connection(_docker: &clients::Cli) -> (C3p0, String) {
    let manager = SqliteConnectionManager::memory();

    let pool = r2d2::Pool::builder().build(manager).unwrap();

    let pool = C3p0Builder::build(pool);

    (pool, "".to_owned())
}
