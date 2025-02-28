use crate::error::into_c3p0_error;
use crate::sqlite::Db;
use c3p0_common::*;
use sqlx::{Pool, Transaction};

#[derive(Clone)]
pub struct SqlxSqliteC3p0Pool {
    pool: Pool<Db>,
}

impl SqlxSqliteC3p0Pool {
    pub fn new(pool: Pool<Db>) -> Self {
        SqlxSqliteC3p0Pool { pool }
    }

    pub fn pool(&self) -> &Pool<Db> {
        &self.pool
    }
}

impl From<Pool<Db>> for SqlxSqliteC3p0Pool {
    fn from(pool: Pool<Db>) -> Self {
        SqlxSqliteC3p0Pool::new(pool)
    }
}

impl C3p0Pool for SqlxSqliteC3p0Pool {
    type Tx<'a> = Transaction<'a, Db>;

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
