use async_trait::async_trait;
use c3p0_common::*;
use futures::Future;

use crate::error::into_c3p0_error;
use sqlx::{Pool, Transaction, Database, Executor, IntoArguments, query::Query};

#[derive(Clone)]
pub struct SqlxC3p0Pool<DB>
    where DB: Clone + Database,
{
    pool: Pool<DB>,
}

impl <DB> SqlxC3p0Pool<DB>
    where DB: Clone + Database,
{
    pub fn new(pool: Pool<DB>) -> Self {
        SqlxC3p0Pool { pool }
    }
}

impl <DB> Into<SqlxC3p0Pool<DB>> for Pool<DB>
    where DB: Clone + Database,
{
    fn into(self) -> SqlxC3p0Pool<DB> {
        SqlxC3p0Pool::new(self)
    }
}

#[async_trait]
impl <DB> C3p0Pool for SqlxC3p0Pool<DB>
    where DB: Clone + Database,
{
    type Conn = SqlxConnection<DB>;

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

pub enum SqlxConnection<DB>
    where DB: Clone + Database,
{
    Tx(&'static mut Transaction<'static, DB>),
}

impl <DB> SqlxConnection<DB>
    where DB: Clone + Database,
{
    pub fn get_conn(&mut self) -> &mut Transaction<'static, DB> {
        match self {
            SqlxConnection::Tx(tx) => tx,
        }
    }

}

pub async fn execute<'q, A, E, DB>(query: Query<'q, DB, A>, executor: E) -> Result<(), C3p0Error>
    where
        DB: Database,
        A: 'q + IntoArguments<'q, DB> + Send,
        E: Executor<'q, Database = DB>,
{
    query
        .execute(executor)
        .await
        .map_err(into_c3p0_error)
        .map(|_| ())
}



#[async_trait]
impl <DB> SqlConnection for SqlxConnection<DB>
    where DB: Clone + Database,
{
    async fn batch_execute(&mut self, sql: &str) -> Result<(), C3p0Error> {
        let query = sqlx::query(sql);
        query
            .execute(self.get_conn())
            .await
            .map_err(into_c3p0_error)
            .map(|_| ())
    }
}
