use crate::deadpool::postgres::{Pool, Transaction};
use crate::tokio_postgres::row::Row;
use crate::tokio_postgres::types::{FromSqlOwned, ToSql};
use crate::*;

use c3p0_common::*;
use std::future::Future;

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
    type Tx = PgTx;

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
        let mut conn = self.pool.get().await.map_err(deadpool_into_c3p0_error)?;

        let native_transaction: Transaction<'_> =
            conn.transaction().await.map_err(into_c3p0_error)?;

        // ToDo: To avoid this unsafe we need GAT
        let mut transaction = PgTx {
            inner: (unsafe {
                ::std::mem::transmute::<
                    &deadpool_postgres::Transaction<'_>,
                    &deadpool_postgres::Transaction<'_>,
                >(&native_transaction)
            }),
        };
        let ref_transaction =
            unsafe { ::std::mem::transmute::<&mut PgTx, &mut PgTx>(&mut transaction) };
        let result = { (tx)(ref_transaction).await? };

        native_transaction.commit().await.map_err(into_c3p0_error)?;

        Ok(result)
    }
}

pub struct PgTx {
    inner: &'static Transaction<'static>,
}

impl PgTx {
    pub async fn batch_execute(&mut self, sql: &str) -> Result<(), C3p0Error> {
        self.inner.batch_execute(sql).await.map_err(into_c3p0_error)
    }

    pub async fn execute(
        &mut self,
        sql: &str,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<u64, C3p0Error> {
        self.inner
            .execute(sql, params)
            .await
            .map_err(into_c3p0_error)
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
        let stmt = self.inner.prepare(sql).await.map_err(into_c3p0_error)?;
        self.inner
            .query(&stmt, params)
            .await
            .map_err(into_c3p0_error)?
            .first()
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
        let stmt = self.inner.prepare(sql).await.map_err(into_c3p0_error)?;
        self.inner
            .query(&stmt, params)
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
