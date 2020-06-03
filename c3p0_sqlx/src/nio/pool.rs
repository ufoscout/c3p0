use async_trait::async_trait;
use c3p0_common::*;
use futures::Future;

use sqlx::{Transaction, Connect, Pool};
use crate::into_c3p0_error;
use sqlx::pool::PoolConnection;

#[derive(Clone)]
pub struct SqlxC3p0PoolAsync<C: Connect + Send + Sync> {
    pool: Pool<C>,
}

impl <C: Connect + Send + Sync> SqlxC3p0PoolAsync<C> {
    pub fn new(pool: Pool<C>) -> Self {
        SqlxC3p0PoolAsync { pool }
    }
}

impl <C: Connect + Send + Sync> Into<SqlxC3p0PoolAsync<C>> for Pool<C> {
    fn into(self) -> SqlxC3p0PoolAsync<C> {
        SqlxC3p0PoolAsync::new(self)
    }
}

#[async_trait]
impl <C: Connect + Send + Sync + Clone> C3p0PoolAsync for SqlxC3p0PoolAsync<C> {
    type Conn = SqlxConnectionAsync<C>;

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
            SqlxConnectionAsync::Tx(unsafe { ::std::mem::transmute(&mut native_transaction) });

        let result = { (tx)(transaction).await? };

        native_transaction.commit().await.map_err(into_c3p0_error)?;

        Ok(result)
    }
}

pub enum SqlxConnectionAsync<C: Connect + Send + Sync> {
    Tx(&'static mut Transaction<PoolConnection<C>>),
}

#[async_trait]
impl <C: Connect + Send + Sync> SqlConnectionAsync for SqlxConnectionAsync<C> {
    async fn batch_execute(&mut self, sql: &str) -> Result<(), C3p0Error> {
        match self {
            SqlxConnectionAsync::Tx(tx) => {
                let query = sqlx::query(sql);
                query.execute(tx).await.map_err(into_c3p0_error).map(|_| ())
            },
        }
    }
}
