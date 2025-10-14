use sqlx::Database;

use crate::error::C3p0Error;

use std::future::Future;

/// A trait for a C3p0 pool.
/// A C3p0 pool is a connection pool for a database.
pub trait C3p0Pool: Clone + Send + Sync {
    /// The DB type.
    type DB: Database;

    /// Creates a new transaction.
    /// It executes the given closure `tx` within a transaction and returns the result of the closure.
    /// if the closure returns an error, the transaction is rolled back and the error is returned,
    /// otherwise the transaction is automatically committed.
    fn transaction<
        T: Send,
        E: Send + From<C3p0Error>,
        F: Send + AsyncFnOnce(&mut <Self::DB as Database>::Connection) -> Result<T, E>,
    >(
        &self,
        tx: F,
    ) -> impl Future<Output = Result<T, E>>;
}
