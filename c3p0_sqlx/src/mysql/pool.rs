use c3p0_common::*;
use std::future::Future;

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
    type Tx = MySqlTx;

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

        // ToDo: To avoid this unsafe we need GAT
        let mut transaction = MySqlTx {
            inner: native_transaction,
        };
        let ref_transaction = unsafe { ::std::mem::transmute(&mut transaction) };

        let result = { (tx)(ref_transaction).await? };

        transaction.inner.commit().await.map_err(into_c3p0_error)?;

        Ok(result)
    }
}

pub struct MySqlTx {
    inner: Transaction<'static, Db>,
}

impl MySqlTx {
    pub fn conn(&mut self) -> &mut MySqlConnection {
        &mut self.inner
    }
}
