use c3p0_common::error::C3p0Error;

use async_trait::async_trait;
use std::future::Future;

#[async_trait(?Send)]
pub trait C3p0PoolAsync: Clone {
    type CONN;

    //    async fn connection(&self) -> Result<Self::CONN, C3p0Error>;

    async fn transaction<
        T,
        E: From<C3p0Error>,
        F: FnOnce(Self::CONN) -> Fut,
        Fut: Future<Output = Result<T, E>>
    >(
        &self,
        tx: F,
    ) -> Result<T, E>;
}

#[async_trait(?Send)]
pub trait SqlConnectionAsync {
    async fn batch_execute(&mut self, sql: &str) -> Result<(), C3p0Error>;
}
