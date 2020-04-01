use c3p0_common::error::C3p0Error;

use async_trait::async_trait;
use std::future::Future;
use std::pin::Pin;

#[async_trait]
pub trait C3p0PoolAsync: Clone {
    type CONN;

    //    async fn connection(&self) -> Result<Self::CONN, C3p0Error>;

    async fn transaction<
        T: Send + Sync,
        E: From<C3p0Error>,
        F: Send + FnOnce(&mut Self::CONN) -> Pin<Box<dyn Future<Output = Result<T, E>> + Send + '_>>,
    >(
        &self,
        tx: F,
    ) -> Result<T, E>;
}

#[async_trait]
pub trait SqlConnectionAsync {
    async fn batch_execute(&mut self, sql: &str) -> Result<(), C3p0Error>;
}
