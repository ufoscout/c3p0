use crate::error::C3p0Error;

use async_trait::async_trait;
use std::future::Future;
use std::pin::Pin;

#[async_trait]
pub trait C3p0Pool: Clone + Send + Sync {
    type Conn;

    //    async fn connection(&self) -> Result<Self::CONN, C3p0Error>;

    async fn transaction<
        T: Send,
        E: Send + From<C3p0Error>,
        F: Send,
    >(
        &self,
        tx: F,
    ) -> Result<T, E>
    where for<'a> F: FnOnce(&'a mut Self::Conn) -> Pin<Box<dyn Future<Output = Result<T, E>> + Send + 'a>>;
}

#[async_trait]
pub trait SqlConnection: Send {
    async fn batch_execute(&mut self, sql: &str) -> Result<(), C3p0Error>;
}
