
use crate::{error::{into_c3p0_error, C3p0Error}, pool::C3p0Pool};
use sqlx::{Pool, Postgres, Transaction};

/// A C3p0Pool implementation for Postgres
#[derive(Clone)]
pub struct PgC3p0Pool {
    pool: Pool<Postgres>,
}

impl PgC3p0Pool {
    /// Creates a new SqlxPgC3p0Pool from a Sqlx Pool
    pub fn new(pool: Pool<Postgres>) -> Self {
        PgC3p0Pool { pool }
    }

    /// Returns the underlying Sqlx Pool
    pub fn pool(&self) -> &Pool<Postgres> {
        &self.pool
    }
}

impl From<Pool<Postgres>> for PgC3p0Pool {
    fn from(pool: Pool<Postgres>) -> Self {
        PgC3p0Pool::new(pool)
    }
}

impl C3p0Pool for PgC3p0Pool {
    type Tx<'a> = Transaction<'a, Postgres>;

    async fn transaction<
        T: Send,
        E: Send + From<C3p0Error>,
        F: Send + AsyncFnOnce(&mut Self::Tx<'_>) -> Result<T, E>,
    >(
        &self,
        tx: F,
    ) -> Result<T, E> {
        let mut transaction = self.pool.begin().await.map_err(into_c3p0_error)?;

        let result = (tx)(&mut transaction).await?;

        transaction.commit().await.map_err(into_c3p0_error)?;
        Ok(result)
    }
}
