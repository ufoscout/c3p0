#![cfg(feature = "in_memory")]

use c3p0::in_memory::*;
use c3p0::*;
use lazy_static::lazy_static;
use maybe_single::MaybeSingle;

pub type C3p0Impl = InMemoryC3p0Pool;

mod tests_json;
pub mod utils;

lazy_static! {
    pub static ref SINGLETON: MaybeSingle<(C3p0Impl, String)> = MaybeSingle::new(|| init());
}

fn init() -> (C3p0Impl, String) {
    let pool = InMemoryC3p0Pool::new();

    (pool, "".to_owned())
}
