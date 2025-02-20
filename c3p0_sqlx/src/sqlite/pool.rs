use crate::error::into_c3p0_error;
use crate::sqlite::Db;
use c3p0_common::*;
use sqlx::{Pool, SqliteConnection, Transaction};

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
    type Tx<'a> = SqliteTx<'a>;

    async fn transaction<
        T: Send,
        E: Send + From<C3p0Error>,
        F: Send + AsyncFnOnce(&mut Self::Tx<'_>) -> Result<T, E>,
    >(
        &self,
        tx: F,
    ) -> Result<T, E> {
        let native_transaction: Transaction<'static, Db> =
            self.pool.begin().await.map_err(into_c3p0_error)?;

        let mut transaction = SqliteTx {
            inner: native_transaction,
        };
        let result = { (tx)(&mut transaction).await? };

        transaction.inner.commit().await.map_err(into_c3p0_error)?;
        Ok(result)
    }
}

pub struct SqliteTx<'a> {
    inner: Transaction<'a, Db>,
}

impl SqliteTx<'_> {
    pub fn conn(&mut self) -> &mut SqliteConnection {
        &mut self.inner
    }
}
