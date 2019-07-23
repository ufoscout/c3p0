use crate::pool::C3p0PoolManager;
use crate::error::C3p0Error;

pub mod error;
pub mod json;
pub mod types;
pub mod pool;

pub struct C3p0Pool<C3P0: C3p0PoolManager>{
    pool_manager: C3P0
}

impl <C3P0: C3p0PoolManager> C3p0Pool<C3P0> {

    pub fn new(pool_manager: C3P0) -> Self {
        C3p0Pool{pool_manager}
    }

    pub fn connection(&self) -> Result<C3P0::CONN, C3p0Error> {
        self.pool_manager.connection()
    }

    pub fn transaction<T, F: Fn(&C3P0::CONN) -> Result<T, Box<std::error::Error>>>(
        &self,
        tx: F,
    ) -> Result<T, C3p0Error> {
        self.pool_manager.transaction(tx)
    }
}