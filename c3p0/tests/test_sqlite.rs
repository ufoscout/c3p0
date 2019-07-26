#![cfg(feature = "sqlite")]

use c3p0::*;
use c3p0::sqlite::r2d2::{Pool, SqliteConnectionManager};
use lazy_static::lazy_static;
use maybe_single::MaybeSingle;
use serde_derive::{Deserialize, Serialize};

pub use c3p0::sqlite::rusqlite::Row;
pub type C3p0Impl = C3p0Pool<SqlitePoolManager>;

mod tests;

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct TestData {
    pub first_name: String,
    pub last_name: String,
}

lazy_static! {
    pub static ref SINGLETON: MaybeSingle<(C3p0Impl, String)> = MaybeSingle::new(|| init());
}

fn init() -> (C3p0Impl, String) {
    let manager = SqliteConnectionManager::memory();

    let pool = Pool::builder().build(manager).unwrap();

    let pool = C3p0Pool::new(SqlitePoolManager::new(pool));

    (pool, "".to_owned())
}
