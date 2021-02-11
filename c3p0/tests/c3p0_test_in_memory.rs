#![cfg(feature = "in_memory")]

use c3p0::in_memory::*;
use c3p0::*;
use maybe_single::nio::{Data, MaybeSingleAsync};
use once_cell::sync::OnceCell;

pub type C3p0Impl = InMemoryC3p0Pool;

mod tests_json;
pub mod utils;

pub type MaybeType = (C3p0Impl, String);

async fn init() -> MaybeType {
    let pool = InMemoryC3p0Pool::new();

    (pool, "".to_owned())
}

pub async fn data(serial: bool) -> Data<'static, MaybeType> {
    static DATA: OnceCell<MaybeSingleAsync<MaybeType>> = OnceCell::new();
    DATA.get_or_init(|| MaybeSingleAsync::new(|| Box::pin(init())))
        .data(serial)
        .await
}

pub mod db_specific {

    use super::*;

    pub fn db_type() -> utils::DbType {
        utils::DbType::InMemory
    }
}
