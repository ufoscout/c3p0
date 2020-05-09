use crate::error::C3p0Error;

use async_trait::async_trait;
use std::future::Future;

#[async_trait(?Send)]
pub trait C3p0PoolAsync: Clone + Send + Sync {
    type Conn: SqlConnectionAsync;

    //    async fn connection(&self) -> Result<Self::CONN, C3p0Error>;

    async fn transaction<
        T,
        E: From<C3p0Error>,
        F: FnOnce(Self::Conn) -> Fut,
        Fut: Future<Output = Result<T, E>>,
    >(
        &self,
        tx: F,
    ) -> Result<T, E>;
}

#[async_trait(?Send)]
pub trait SqlConnectionAsync: Send {
    async fn batch_execute(&mut self, sql: &str) -> Result<(), C3p0Error>;
}
