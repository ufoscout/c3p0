use async_trait::async_trait;
use c3p0_common::*;
use std::future::Future;

use crate::common::executor::batch_execute;
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

#[async_trait]
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
        let mut native_transaction: Transaction<'_, Db> =
            self.pool.begin().await.map_err(into_c3p0_error)?;

        // ToDo: To avoid this unsafe we need GAT
        let mut transaction = MySqlTx {
            inner: unsafe { ::std::mem::transmute(&mut native_transaction) },
        };
        let ref_transaction = unsafe { ::std::mem::transmute(&mut transaction) };

        let result = { (tx)(ref_transaction).await? };

        native_transaction.commit().await.map_err(into_c3p0_error)?;

        Ok(result)
    }
}

pub struct MySqlTx {
    inner: &'static mut Transaction<'static, Db>,
}

impl MySqlTx {
    pub fn conn(&mut self) -> &mut MySqlConnection {
        self.inner
    }
}

#[async_trait]
impl SqlTx for MySqlTx {
    async fn batch_execute(&mut self, sql: &str) -> Result<(), C3p0Error> {
        batch_execute(sql, self.conn()).await
    }
}
