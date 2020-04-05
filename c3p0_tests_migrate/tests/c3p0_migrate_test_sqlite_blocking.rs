#![cfg(feature = "sqlite_blocking")]

pub use c3p0::blocking::*;
use c3p0::sqlite::blocking::r2d2::{Pool, SqliteConnectionManager};
pub use c3p0::sqlite::blocking::*;
use testcontainers::*;

mod tests_blocking;
pub mod utils;

pub fn new_connection(_docker: &clients::Cli) -> (SqliteC3p0Pool, String) {
    let manager = SqliteConnectionManager::memory();

    let pool = Pool::builder().build(manager).unwrap();

    let pool = SqliteC3p0Pool::new(pool);

    (pool, "".to_owned())
}

pub mod db_specific {
    use super::*;

    pub fn db_type() -> utils::DbType {
        utils::DbType::Sqlite
    }
}