#![cfg(feature = "in_memory_blocking")]

use c3p0::in_memory::blocking::*;
use c3p0::blocking::*;
use lazy_static::lazy_static;
use maybe_single::{Data, MaybeSingle};

pub type C3p0Impl = InMemoryC3p0Pool;

mod tests_blocking_json;
pub mod utils;

pub type MaybeType = (C3p0Impl, String);

lazy_static! {
    pub static ref SINGLETON: MaybeSingle<MaybeType> = MaybeSingle::new(|| init());
}

fn init() -> MaybeType {
    let pool = InMemoryC3p0Pool::new();

    (pool, "".to_owned())
}

pub fn data(serial: bool) -> Data<'static, MaybeType> {
    SINGLETON.data(serial)
}


pub mod db_specific {

    use super::*;

    pub fn db_type() -> utils::DbType {
        utils::DbType::InMemory
    }

}