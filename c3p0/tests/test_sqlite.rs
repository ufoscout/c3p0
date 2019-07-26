#![cfg(feature = "sqlite")]

use c3p0_json::sqlite::r2d2::{Pool, SqliteConnectionManager};
use lazy_static::lazy_static;
use maybe_single::MaybeSingle;
use serde_derive::{Deserialize, Serialize};

pub use c3p0_json::sqlite::rusqlite::Row;
pub use c3p0_json::sqlite::SqlitePoolManager as C3p0Impl;
pub use c3p0_json::sqlite::C3p0SqliteBuilder as C3p0BuilderImpl;
pub use c3p0_json::C3p0SqliteJson as C3p0JsonImpl;
pub use c3p0_json::C3p0SqliteJsonBuilder as C3p0JsonBuilderImpl;

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

    let pool = C3p0BuilderImpl::build(pool);

    (pool, "".to_owned())
}
