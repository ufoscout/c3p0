use crate::error::C3p0Error;

use async_trait::async_trait;
use std::future::Future;

#[async_trait]
pub trait C3p0Pool: Clone + Send + Sync {
    type Conn;

    //    async fn connection(&self) -> Result<Self::CONN, C3p0Error>;

    async fn transaction<
        'a,
        T: Send,
        E: Send + From<C3p0Error>,
        F: Send + FnOnce(&'a mut Self::Conn) -> Fut,
        Fut: Send + Future<Output = Result<T, E>>,
    >(
        &'a self,
        tx: F,
    ) -> Result<T, E>;
}

#[async_trait]
pub trait SqlConnection: Send {
    async fn batch_execute(&mut self, sql: &str) -> Result<(), C3p0Error>;
}
