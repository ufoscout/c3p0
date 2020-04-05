#![cfg(feature = "in_memory_blocking")]

use c3p0::blocking::*;
use c3p0::in_memory::blocking::*;
use maybe_single::{Data, MaybeSingle};
use once_cell::sync::OnceCell;

pub type C3p0Impl = InMemoryC3p0Pool;

mod tests_blocking_json;
pub mod utils;

pub type MaybeType = (C3p0Impl, String);


fn init() -> MaybeType {
    let pool = InMemoryC3p0Pool::new();

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
        utils::DbType::InMemory
    }
}
