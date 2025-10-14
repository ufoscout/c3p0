use crate::error::C3p0Error;
use crate::{error::into_c3p0_error, pool::C3p0Pool};
use sqlx::{Pool, Sqlite, SqliteConnection};

/// A C3p0Pool implementation for Sqlite
#[derive(Clone)]
pub struct SqliteC3p0Pool {
    pool: Pool<Sqlite>,
}

impl SqliteC3p0Pool {
    /// Creates a new SqlxSqliteC3p0Pool from a Sqlx Pool
    pub fn new(pool: Pool<Sqlite>) -> Self {
        SqliteC3p0Pool { pool }
    }

    /// Returns the underlying Sqlx Pool
    pub fn pool(&self) -> &Pool<Sqlite> {
        &self.pool
    }
}

impl From<Pool<Sqlite>> for SqliteC3p0Pool {
    fn from(pool: Pool<Sqlite>) -> Self {
        SqliteC3p0Pool::new(pool)
    }
}

impl C3p0Pool for SqliteC3p0Pool {
    type DB = Sqlite;

    async fn transaction<
        T: Send,
        E: Send + From<C3p0Error>,
        F: Send + AsyncFnOnce(&mut SqliteConnection) -> Result<T, E>,
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
