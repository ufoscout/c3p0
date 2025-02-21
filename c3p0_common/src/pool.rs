use crate::error::C3p0Error;

use std::future::Future;

pub trait C3p0Pool: Clone + Send + Sync {
    type Tx<'a>;

    fn transaction<
        T: Send,
        E: Send + From<C3p0Error>,
        F: Send + AsyncFnOnce(&mut Self::Tx<'_>) -> Result<T, E>,
    >(
        &self,
        tx: F,
    ) -> impl Future<Output = Result<T, E>>;
}
