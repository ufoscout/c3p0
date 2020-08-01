use async_trait::async_trait;
use c3p0_common::*;
use futures::Future;

use crate::{into_c3p0_error, Db};
use sqlx::{Pool, Transaction};

#[derive(Clone)]
pub struct SqlxC3p0Pool {
    pool: Pool<Db>,
}

impl SqlxC3p0Pool {
    pub fn new(pool: Pool<Db>) -> Self {
        SqlxC3p0Pool { pool }
    }
}

impl Into<SqlxC3p0Pool> for Pool<Db> {
    fn into(self) -> SqlxC3p0Pool {
        SqlxC3p0Pool::new(self)
    }
}

#[async_trait]
impl C3p0Pool for SqlxC3p0Pool {
    type Conn = SqlxConnection;

    async fn transaction<
        T: Send,
        E: Send + From<C3p0Error>,
        F: Send + FnOnce(Self::Conn) -> Fut,
        Fut: Send + Future<Output = Result<T, E>>,
    >(
        &self,
        tx: F,
    ) -> Result<T, E> {
        let mut native_transaction = self.pool.begin().await.map_err(into_c3p0_error)?;

        // ToDo: To avoid this unsafe we need GAT
        let transaction =
            SqlxConnection::Tx(unsafe { ::std::mem::transmute(&mut native_transaction) });

        let result = { (tx)(transaction).await? };

        native_transaction.commit().await.map_err(into_c3p0_error)?;

        Ok(result)
    }
}

pub enum SqlxConnection {
    Tx(&'static mut Transaction<'static, Db>),
}

impl SqlxConnection {
    pub fn get_conn(&mut self) -> &mut Transaction<'static, Db> {
        match self {
            SqlxConnection::Tx(tx) => tx,
        }
    }
}

#[async_trait]
impl SqlConnection for SqlxConnection {
    async fn batch_execute(&mut self, sql: &str) -> Result<(), C3p0Error> {
        let query = sqlx::query(sql);
        query
            .execute(self.get_conn())
            .await
            .map_err(into_c3p0_error)
            .map(|_| ())
    }
}
