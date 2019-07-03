#![cfg(feature = "sqlite")]

use c3p0::prelude::*;
use lazy_static::lazy_static;
use maybe_single::MaybeSingle;
use r2d2_sqlite::SqliteConnectionManager;
use serde_derive::{Deserialize, Serialize};

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
