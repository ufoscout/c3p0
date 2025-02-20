use c3p0_common::*;

use crate::error::into_c3p0_error;
use crate::mysql::Db;
use sqlx::{MySqlConnection, Pool, Transaction};

#[derive(Clone)]
pub struct SqlxMySqlC3p0Pool {
    pool: Pool<Db>,
}

impl SqlxMySqlC3p0Pool {
    pub fn new(pool: Pool<Db>) -> Self {
        SqlxMySqlC3p0Pool { pool }
    }

    pub fn pool(&self) -> &Pool<Db> {
        &self.pool
    }
}

impl From<Pool<Db>> for SqlxMySqlC3p0Pool {
    fn from(pool: Pool<Db>) -> Self {
        SqlxMySqlC3p0Pool::new(pool)
    }
}

impl C3p0Pool for SqlxMySqlC3p0Pool {
    type Tx<'a> = MySqlTx<'a>;

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

        let mut transaction = MySqlTx {
            inner: native_transaction,
        };

        let result = { (tx)(&mut transaction).await? };

        transaction.inner.commit().await.map_err(into_c3p0_error)?;

        Ok(result)
    }
}

pub struct MySqlTx<'a> {
    inner: Transaction<'a, Db>,
}

impl MySqlTx<'_> {
    pub fn conn(&mut self) -> &mut MySqlConnection {
        &mut self.inner
    }
}
