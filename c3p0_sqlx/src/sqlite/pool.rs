use crate::common::executor::batch_execute;
use crate::error::into_c3p0_error;
use crate::sqlite::Db;
use async_trait::async_trait;
use c3p0_common::*;
use sqlx::{Pool, Transaction};
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
    type Conn = SqlxSqliteConnection;

    async fn transaction<
        'a,
        T: Send,
        E: Send + From<C3p0Error>,
        F: Send + FnOnce(&'a mut Self::Conn) -> Fut,
        Fut: Send + Future<Output = Result<T, E>>,
    >(
        &'a self,
        tx: F,
    ) -> Result<T, E> {
        let mut native_transaction: Transaction<'_, Db> =
            self.pool.begin().await.map_err(into_c3p0_error)?;

        // ToDo: To avoid this unsafe we need GAT
        let mut transaction = SqlxSqliteConnection {
            inner: unsafe { ::std::mem::transmute(&mut native_transaction) },
        };
        let ref_transaction = unsafe { ::std::mem::transmute(&mut transaction) };
        let result = { (tx)(ref_transaction).await? };

        native_transaction.commit().await.map_err(into_c3p0_error)?;
        Ok(result)
    }
}

pub struct SqlxSqliteConnection {
    inner: &'static mut Transaction<'static, Db>,
}

impl SqlxSqliteConnection {
    pub fn get_conn(&mut self) -> &mut Transaction<'static, Db> {
        self.inner
    }
}

#[async_trait]
impl SqlConnection for SqlxSqliteConnection {
    async fn batch_execute(&mut self, sql: &str) -> Result<(), C3p0Error> {
        batch_execute(sql, &mut **self.get_conn()).await
    }
}
