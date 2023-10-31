use async_trait::async_trait;
use c3p0_common::*;
use std::future::Future;

use crate::common::executor::batch_execute;
use crate::error::into_c3p0_error;
use crate::postgres::Db;
use sqlx::{Pool, Transaction};

#[derive(Clone)]
pub struct SqlxPgC3p0Pool {
    pool: Pool<Db>,
}

impl SqlxPgC3p0Pool {
    pub fn new(pool: Pool<Db>) -> Self {
        SqlxPgC3p0Pool { pool }
    }
}

impl From<Pool<Db>> for SqlxPgC3p0Pool {
    fn from(pool: Pool<Db>) -> Self {
        SqlxPgC3p0Pool::new(pool)
    }
}

#[async_trait]
impl C3p0Pool for SqlxPgC3p0Pool {
    type Conn = SqlxPgConnection;

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
        let mut native_transaction = self.pool.begin().await.map_err(into_c3p0_error)?;

        // ToDo: To avoid this unsafe we need GAT
        let mut transaction = SqlxPgConnection::Tx(unsafe { ::std::mem::transmute(&mut native_transaction) });
        let ref_transaction = unsafe { 
            ::std::mem::transmute(&mut transaction) 
        };

        let result = { (tx)(ref_transaction).await? };

        native_transaction.commit().await.map_err(into_c3p0_error)?;

        Ok(result)
    }
}

pub enum SqlxPgConnection {
    Tx(&'static mut Transaction<'static, Db>),
}

impl SqlxPgConnection {
    pub fn get_conn(&mut self) -> &mut Transaction<'static, Db> {
        match self {
            SqlxPgConnection::Tx(tx) => tx,
        }
    }
}

#[async_trait]
impl SqlConnection for SqlxPgConnection {
    async fn batch_execute(&mut self, sql: &str) -> Result<(), C3p0Error> {
        batch_execute(sql, &mut **self.get_conn()).await
    }
}
