use c3p0_common::error::C3p0Error;

use async_trait::async_trait;
use std::future::Future;

#[async_trait]
pub trait C3p0PoolAsync: Clone + Send + Sync {
    type CONN: SqlConnectionAsync;

    //    async fn connection(&self) -> Result<Self::CONN, C3p0Error>;

    async fn transaction<
        T: Send,
        E: Send + From<C3p0Error>,
        F: Send + FnOnce(Self::CONN) -> Fut,
        Fut: Send + Future<Output = Result<T, E>>,
    >(
        &self,
        tx: F,
    ) -> Result<T, E>;
}

#[async_trait]
pub trait SqlConnectionAsync: Send {
    async fn batch_execute(&mut self, sql: &str) -> Result<(), C3p0Error>;
}
