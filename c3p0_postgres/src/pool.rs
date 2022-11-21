use crate::deadpool::postgres::Pool;
use crate::tokio_postgres::row::Row;
use crate::tokio_postgres::types::{FromSqlOwned, ToSql};
use crate::*;

use c3p0_common::*;
use deadpool_postgres::Transaction;
use std::future::Future;

pub enum PgC3p0ConnectionManager {
    DeadPool,
}

impl PgC3p0ConnectionManager {}

#[derive(Clone)]
pub struct PgC3p0Pool {
    pool: Pool,
}

impl PgC3p0Pool {
    pub fn new(pool: Pool) -> Self {
        PgC3p0Pool { pool }
    }
}

impl From<Pool> for PgC3p0Pool {
    fn from(pool: Pool) -> Self {
        PgC3p0Pool::new(pool)
    }
}

impl C3p0Pool for PgC3p0Pool {
    type Conn = PgConnection;

    async fn transaction<
        T: Send,
        E: Send + From<C3p0Error>,
        F: FnOnce(&mut Self::Conn) -> Fut,
        Fut: Future<Output = Result<T, E>>,
    >(
        &self,
        tx: F,
    ) -> Result<T, E> {
        let mut conn = self.pool.get().await.map_err(deadpool_into_c3p0_error)?;

        let native_transaction = conn.transaction().await.map_err(into_c3p0_error)?;

        // ToDo: To avoid this unsafe we need GAT
        // See: https://github.com/rust-lang/rust/issues/104678
        // and https://users.rust-lang.org/t/lifetime-ignored-by-async-trait-fn/84517
        let native_transaction: Transaction<'static> = unsafe { ::std::mem::transmute(native_transaction) };

        let mut transaction = PgConnection{tx: native_transaction};

        let result = { (tx)(&mut transaction).await? };

        transaction.tx.commit().await.map_err(into_c3p0_error)?;

        Ok(result)
    }
}

pub struct PgConnection {
    tx: Transaction<'static>,
}

// impl <'a> SqlConnection<'a> for PgConnection<'a> {
//     async fn batch_execute(&mut self, sql: &str) -> Result<(), C3p0Error> {
//         match self {
//             PgConnection::Tx(tx) => tx.batch_execute(sql).await.map_err(into_c3p0_error),
//         }
//     }
// }

impl PgConnection {
    pub async fn execute(
        &mut self,
        sql: &str,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<u64, C3p0Error> {
        self.tx.execute(sql, params).await.map_err(into_c3p0_error)
    }

    pub async fn fetch_one_value<T: FromSqlOwned>(
        &mut self,
        sql: &str,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<T, C3p0Error> {
        self.fetch_one(sql, params, to_value_mapper).await
    }

    pub async fn fetch_one<T, F: Fn(&Row) -> Result<T, Box<dyn std::error::Error>>>(
        &mut self,
        sql: &str,
        params: &[&(dyn ToSql + Sync)],
        mapper: F,
    ) -> Result<T, C3p0Error> {
        self.fetch_one_optional(sql, params, mapper)
            .await
            .and_then(|result| result.ok_or(C3p0Error::ResultNotFoundError))
    }

    pub async fn fetch_one_optional<T, F: Fn(&Row) -> Result<T, Box<dyn std::error::Error>>>(
        &mut self,
        sql: &str,
        params: &[&(dyn ToSql + Sync)],
        mapper: F,
    ) -> Result<Option<T>, C3p0Error> {
        let stmt = self.tx.prepare(sql).await.map_err(into_c3p0_error)?;
                self.tx.query(&stmt, params)
                    .await
                    .map_err(into_c3p0_error)?
                    .get(0)
                    .map(mapper)
                    .transpose()
                    .map_err(|err| C3p0Error::RowMapperError {
                        cause: format!("{:?}", err),
                    })
    }

    pub async fn fetch_all<T, F: Fn(&Row) -> Result<T, Box<dyn std::error::Error>>>(
        &mut self,
        sql: &str,
        params: &[&(dyn ToSql + Sync)],
        mapper: F,
    ) -> Result<Vec<T>, C3p0Error> {
                let stmt = self.tx.prepare(sql).await.map_err(into_c3p0_error)?;
                self.tx.query(&stmt, params)
                    .await
                    .map_err(into_c3p0_error)?
                    .iter()
                    .map(mapper)
                    .collect::<Result<Vec<T>, Box<dyn std::error::Error>>>()
                    .map_err(|err| C3p0Error::RowMapperError {
                        cause: format!("{:?}", err),
                    })
    }

    pub async fn fetch_all_values<T: FromSqlOwned>(
        &mut self,
        sql: &str,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<Vec<T>, C3p0Error> {
        self.fetch_all(sql, params, to_value_mapper).await
    }
}
