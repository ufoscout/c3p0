use crate::error::C3p0Error;
use crate::{error::into_c3p0_error, pool::C3p0Pool};
use sqlx::{MySql, Pool, Transaction};

/// A C3p0Pool implementation for MySql
#[derive(Clone)]
pub struct MySqlC3p0Pool {
    pool: Pool<MySql>,
}

impl MySqlC3p0Pool {
    /// Creates a new SqlxMySqlC3p0Pool from a Sqlx Pool
    pub fn new(pool: Pool<MySql>) -> Self {
        MySqlC3p0Pool { pool }
    }

    /// Returns the underlying Sqlx Pool
    pub fn pool(&self) -> &Pool<MySql> {
        &self.pool
    }
}

impl From<Pool<MySql>> for MySqlC3p0Pool {
    fn from(pool: Pool<MySql>) -> Self {
        MySqlC3p0Pool::new(pool)
    }
}

impl C3p0Pool for MySqlC3p0Pool {
    type Tx<'a> = Transaction<'a, MySql>;

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
