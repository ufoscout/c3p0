use c3p0_common::*;
use std::future::Future;

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
    type Tx = PgTx;

    async fn transaction<
        'a,
        T: Send,
        E: Send + From<C3p0Error>,
        F: Send + FnOnce(&'a mut Self::Tx) -> Fut,
        Fut: Send + Future<Output = Result<T, E>>,
    >(
        &'a self,
        tx: F,
    ) -> Result<T, E> {
        let native_transaction: Transaction<'static, Db> =
            self.pool.begin().await.map_err(into_c3p0_error)?;

        let mut transaction = PgTx {
            inner: native_transaction,
        };

        // ToDo: To avoid this unsafe we need GAT
        let ref_transaction =
            unsafe { ::std::mem::transmute::<&mut PgTx, &mut PgTx>(&mut transaction) };

        let result = { (tx)(ref_transaction).await? };

        transaction.inner.commit().await.map_err(into_c3p0_error)?;

        Ok(result)
    }
}

pub struct PgTx {
    inner: Transaction<'static, Db>,
}

impl PgTx {
    pub fn conn(&mut self) -> &mut PgConnection {
        &mut self.inner
    }
}
