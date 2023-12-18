use crate::common::executor::batch_execute;
use crate::error::into_c3p0_error;
use crate::sqlite::Db;
use async_trait::async_trait;
use c3p0_common::*;
use sqlx::{Pool, SqliteConnection, Transaction};
use std::future::Future;

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

#[async_trait]
impl C3p0Pool for SqlxSqliteC3p0Pool {
    type Tx = SqliteTx;

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
        let mut transaction = SqliteTx {
            inner: native_transaction,
        };
        let ref_transaction = unsafe { ::std::mem::transmute(&mut transaction) };
        let result = { (tx)(ref_transaction).await? };

        transaction.inner.commit().await.map_err(into_c3p0_error)?;
        Ok(result)
    }
}

pub struct SqliteTx {
    inner: Transaction<'static, Db>,
}

impl SqliteTx {
    pub fn conn(&mut self) -> &mut SqliteConnection {
        &mut self.inner
    }
}

#[async_trait]
impl SqlTx for SqliteTx {
    async fn batch_execute(&mut self, sql: &str) -> Result<(), C3p0Error> {
        batch_execute(sql, self.conn()).await
    }
}
