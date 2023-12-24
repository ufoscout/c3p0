use crate::error::C3p0Error;

use async_trait::async_trait;
use std::future::Future;

#[async_trait]
pub trait C3p0Pool: Clone + Send + Sync {
    type Tx;

    //    async fn connection(&self) -> Result<Self::CONN, C3p0Error>;

    async fn transaction<
        'a,
        T: Send,
        E: Send + From<C3p0Error>,
        F: Send + FnOnce(&'a mut Self::Tx) -> Fut,
        Fut: Send + Future<Output = Result<T, E>>,
    >(
        &'a self,
        tx: F,
    ) -> Result<T, E>;
}
