#![cfg(feature = "sqlite")]

use c3p0::sqlite::r2d2::{Pool, SqliteConnectionManager};
use c3p0::sqlite::*;
use c3p0::*;
use lazy_static::lazy_static;
use maybe_single::MaybeSingle;

pub use c3p0::sqlite::driver::Row;
pub type C3p0Impl = SqliteC3p0Pool;

mod tests;
mod tests_json;
pub mod utils;

lazy_static! {
    pub static ref SINGLETON: MaybeSingle<(C3p0Impl, String)> = MaybeSingle::new(|| init());
}

fn init() -> (C3p0Impl, String) {
    let manager = SqliteConnectionManager::memory();

    let pool = Pool::builder().build(manager).unwrap();

    let pool = SqliteC3p0Pool::new(pool);

    (pool, "".to_owned())
}
