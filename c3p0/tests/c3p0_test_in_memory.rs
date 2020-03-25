#![cfg(feature = "in_memory")]

use c3p0::in_memory::*;
use c3p0::*;
use lazy_static::lazy_static;
use maybe_single::{Data, MaybeSingle};

pub type C3p0Impl = InMemoryC3p0Pool;

mod tests_json;
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
