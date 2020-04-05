#![cfg(feature = "sqlite_blocking")]

use c3p0::blocking::*;
use c3p0::sqlite::blocking::r2d2::{Pool, SqliteConnectionManager};
pub use c3p0::sqlite::blocking::rusqlite::Row;
use c3p0::sqlite::blocking::*;
use maybe_single::{Data, MaybeSingle};
use once_cell::sync::OnceCell;

pub type C3p0Impl = SqliteC3p0Pool;

mod tests_blocking;
mod tests_blocking_json;
mod utils;

pub type MaybeType = (C3p0Impl, String);

fn init() -> MaybeType {
    let manager = SqliteConnectionManager::memory();

    let pool = Pool::builder().build(manager).unwrap();

    let pool = SqliteC3p0Pool::new(pool);

    (pool, "".to_owned())
}

pub fn data(serial: bool) -> Data<'static, MaybeType> {
    static DATA: OnceCell<MaybeSingle<MaybeType>> = OnceCell::new();
    DATA.get_or_init(|| MaybeSingle::new(|| init()))
        .data(serial)
}

pub mod db_specific {

    use super::*;

    pub fn db_type() -> utils::DbType {
        utils::DbType::Sqlite
    }

    pub fn row_to_string(row: &Row) -> Result<String, Box<dyn std::error::Error>> {
        Ok(row.get(0)?)
    }

    pub fn build_insert_query(table_name: &str) -> String {
        format!(r"INSERT INTO {} (name) VALUES (?)", table_name)
    }
}
