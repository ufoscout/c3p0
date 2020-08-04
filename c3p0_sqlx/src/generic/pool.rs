use async_trait::async_trait;
use c3p0_common::*;
use futures::Future;

use crate::error::into_c3p0_error;
use std::ops::{Deref, DerefMut};
use sqlx::{Pool, Transaction, Database, Executor, IntoArguments, query::Query};
use sqlx::database::HasArguments;

#[derive(Clone)]
pub struct SqlxC3p0Pool<DB>
    where DB: Clone + Database,
        for<'c> &'c sqlx::Transaction<'c, DB> : sqlx::Executor<'c, Database = DB>,
{
    pool: Pool<DB>,
}

impl <DB> SqlxC3p0Pool<DB>
    where DB: Clone + Database,
          for<'c> &'c sqlx::Transaction<'c, DB> : sqlx::Executor<'c, Database = DB>,
{
    pub fn new(pool: Pool<DB>) -> Self {
        SqlxC3p0Pool { pool }
    }
}

impl <DB> Into<SqlxC3p0Pool<DB>> for Pool<DB>
    where DB: Clone + Database,
          for<'c> &'c sqlx::Transaction<'c, DB> : sqlx::Executor<'c, Database = DB>,
{
    fn into(self) -> SqlxC3p0Pool<DB> {
        SqlxC3p0Pool::new(self)
    }
}

#[async_trait]
impl <DB> C3p0Pool for SqlxC3p0Pool<DB>
    where DB: Clone + Database,
          for<'c> <DB as sqlx::database::HasArguments<'c>>::Arguments: sqlx::IntoArguments<'c, DB>,
          for<'c> &'c sqlx::Transaction<'c, DB> : sqlx::Executor<'c, Database = DB>,
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
            SqlxConnection{tx: unsafe { ::std::mem::transmute(&mut native_transaction) }};

        let result = { (tx)(transaction).await? };

        native_transaction.commit().await.map_err(into_c3p0_error)?;

        Ok(result)
    }
}

pub struct  SqlxConnection<DB>
    where DB: Clone + Database,
          for<'c> &'c sqlx::Transaction<'c, DB> : sqlx::Executor<'c, Database = DB>
{
    tx: &'static mut Transaction<'static, DB>
}

impl <DB> Deref for SqlxConnection<DB>
    where DB: Clone + Database,
          for<'c> &'c sqlx::Transaction<'c, DB> : sqlx::Executor<'c, Database = DB>
{
    type Target = Transaction<'static, DB>;

    fn deref(&self) -> &Self::Target {
        &self.tx
    }
}

impl <DB> DerefMut for SqlxConnection<DB>
    where DB: Clone + Database,
          for<'c> &'c sqlx::Transaction<'c, DB> : sqlx::Executor<'c, Database = DB>
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.tx
    }
}

pub async fn execute<'q, A, E, DB>(query: Query<'q, DB, A>, executor: E) -> Result<(), C3p0Error>
    where
        DB: Database,
        A: 'q + IntoArguments<'q, DB>,
        E: Executor<'q, Database = DB>,
{
    query
        .execute(executor)
        .await
        .map_err(into_c3p0_error)
        .map(|_| ())
}

pub async fn execute_2<'q, E, DB>(query: Query<'q, DB, <DB as HasArguments<'q>>::Arguments>, executor: E) -> Result<(), C3p0Error>
    where
        DB: Database,
        <DB as sqlx::database::HasArguments<'q>>::Arguments: sqlx::IntoArguments<'q, DB>,
        E: Executor<'q, Database = DB>,
{
    query
        .execute(executor)
        .await
        .map_err(into_c3p0_error)
        .map(|_| ())
}



impl <DB> SqlxConnection<DB>
    where DB: Clone + Database,
          for<'c> <DB as sqlx::database::HasArguments<'c>>::Arguments: sqlx::IntoArguments<'c, DB>,
    //E: Executor<'q, Database = DB>,
          for<'c> &'c sqlx::Transaction<'c, DB> : sqlx::Executor<'c, Database = DB>,

          for<'c> sqlx::Transaction<'c, DB> : sqlx::Executor<'c, Database = DB>,
// for<'c> sqlx:Executor<'c>::Database as sqlx::database::HasArguments<'c>>::Arguments: sqlx::IntoArguments<'c, DB>,
{

    pub fn as_executor(&self) -> &Transaction<'static, DB> {
        self.deref()
    }

    async fn batch_execute(&mut self, sql: &str) -> Result<(), C3p0Error> {
        let query  = sqlx::query::<DB>(sql);
        execute(query, self.as_executor()).await

        // query
        //     .execute(&*self.tx)
        //     .await
        //     .map_err(into_c3p0_error)?;
        // Ok(())
    }
}