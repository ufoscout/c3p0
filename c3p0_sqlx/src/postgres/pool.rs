use c3p0_common::*;

use crate::error::into_c3p0_error;
use crate::postgres::Db;
use sqlx::{PgConnection, Pool, Transaction};

#[derive(Clone)]
pub struct SqlxPgC3p0Pool {
    pool: Pool<Db>,
}

impl SqlxPgC3p0Pool {
    pub fn new(pool: Pool<Db>) -> Self {
        SqlxPgC3p0Pool { pool }
    }

    pub fn pool(&self) -> &Pool<Db> {
        &self.pool
    }
}

impl From<Pool<Db>> for SqlxPgC3p0Pool {
    fn from(pool: Pool<Db>) -> Self {
        SqlxPgC3p0Pool::new(pool)
    }
}

impl C3p0Pool for SqlxPgC3p0Pool {
    type Tx<'a> = PgTx<'a>;

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

        let mut transaction = PgTx {
            inner: native_transaction,
        };

        let result = { (tx)(&mut transaction).await? };

        transaction.inner.commit().await.map_err(into_c3p0_error)?;

        Ok(result)
    }
}

pub struct PgTx<'a> {
    inner: Transaction<'a, Db>,
}

impl PgTx<'_> {
    pub fn conn(&mut self) -> &mut PgConnection {
        &mut self.inner
    }
}
