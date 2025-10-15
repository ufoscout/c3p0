use crate::{
    error::C3p0Error,
    pool::C3p0Pool,
};
use sqlx::{PgConnection, Pool, Postgres};

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
    type DB = Postgres;

    async fn transaction<
        T: Send,
        E: Send + From<C3p0Error>,
        F: Send + AsyncFnOnce(&mut PgConnection) -> Result<T, E>,
    >(
        &self,
        tx: F,
    ) -> Result<T, E> {
        let mut transaction = self.pool.begin().await.map_err(C3p0Error::from)?;

        let result = (tx)(&mut transaction).await?;

        transaction.commit().await.map_err(C3p0Error::from)?;
        Ok(result)
    }
}
