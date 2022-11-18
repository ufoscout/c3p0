use crate::error::C3p0Error;

use std::future::Future;

pub trait C3p0Pool: Clone + Send + Sync {
    type Conn;

    //    async fn connection(&self) -> Result<Self::CONN, C3p0Error>;

    async fn transaction<
        T: Send,
        E: Send + From<C3p0Error>,
        F: Send + FnOnce(Self::Conn) -> Fut,
        Fut: Future<Output = Result<T, E>>,
    >(
        &self,
        tx: F,
    ) -> Result<T, E>;
}

pub trait SqlConnection: Send {
    async fn batch_execute(&mut self, sql: &str) -> Result<(), C3p0Error>;
}
