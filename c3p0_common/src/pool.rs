use crate::error::C3p0Error;

use std::future::Future;

pub trait C3p0Pool: Clone + Send + Sync {
    type Tx;

    fn transaction<
        'a,
        T: Send,
        E: Send + From<C3p0Error>,
        F: Send + FnOnce(&'a mut Self::Tx) -> Fut,
        Fut: Send + Future<Output = Result<T, E>>,
    >(
        &'a self,
        tx: F,
    ) -> impl Future<Output = Result<T, E>> + Send;
}
