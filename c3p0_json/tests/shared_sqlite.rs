#![cfg(feature = "sqlite")]

use lazy_static::lazy_static;
use maybe_single::MaybeSingle;
use r2d2_sqlite::SqliteConnectionManager;
use serde_derive::{Deserialize, Serialize};

pub use c3p0_json::C3p0Sqlite as C3p0;
pub use c3p0_json::C3p0SqliteBuilder as C3p0Builder;
pub use c3p0_json::C3p0SqliteJson as C3p0Json;
pub use c3p0_json::C3p0SqliteJsonBuilder as C3p0JsonBuilder;
pub use rusqlite::Row;

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct TestData {
    pub first_name: String,
    pub last_name: String,
}

lazy_static! {
    pub static ref SINGLETON: MaybeSingle<(C3p0, String)> = MaybeSingle::new(|| init());
}

fn init() -> (C3p0, String) {
    let manager = SqliteConnectionManager::memory();

    let pool = r2d2::Pool::builder().build(manager).unwrap();

    let pool = C3p0Builder::build(pool);

    (pool, "".to_owned())
}
