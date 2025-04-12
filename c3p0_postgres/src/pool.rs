use crate::deadpool::postgres::{Pool, Transaction};
use crate::tokio_postgres::row::Row;
use crate::tokio_postgres::types::{FromSqlOwned, ToSql};
use crate::*;

use c3p0_common::*;

/// A C3p0Pool implementation for Postgres
#[derive(Clone)]
pub struct PgC3p0Pool {
    pool: Pool,
}

impl PgC3p0Pool {
    /// Creates a new PgC3p0Pool from a deadpool Pool
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
    type Tx<'a> = PgTx<'a>;

    async fn transaction<
        T: Send,
        E: Send + From<C3p0Error>,
        F: Send + AsyncFnOnce(&mut Self::Tx<'_>) -> Result<T, E>,
    >(
        &self,
        tx: F,
    ) -> Result<T, E> {
        let mut conn = self.pool.get().await.map_err(deadpool_into_c3p0_error)?;

        let native_transaction = conn.transaction().await.map_err(into_c3p0_error)?;

        let mut transaction = PgTx {
            inner: native_transaction,
        };

        let result = { (tx)(&mut transaction).await? };

        transaction.inner.commit().await.map_err(into_c3p0_error)?;

        Ok(result)
    }
}

pub struct PgTx<'a> {
    inner: Transaction<'a>,
}

impl PgTx<'_> {
    /// Executes a batch of SQL statements
    pub async fn batch_execute(&mut self, sql: &str) -> Result<(), C3p0Error> {
        self.inner.batch_execute(sql).await.map_err(into_c3p0_error)
    }

    /// Executes a SQL statement
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

    /// Fetches a single value with a custom sql query and parameters
    pub async fn fetch_one_value<T: FromSqlOwned>(
        &mut self,
        sql: &str,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<T, C3p0Error> {
        self.fetch_one(sql, params, to_value_mapper).await
    }

    /// Fetches a single row with a custom sql query and parameters
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

    /// Fetches a single row with a custom sql query and parameters
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

    /// Fetches multiple rows with a custom sql query and parameters
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

    /// Fetches multiple values with a custom sql query and parameters
    pub async fn fetch_all_values<T: FromSqlOwned>(
        &mut self,
        sql: &str,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<Vec<T>, C3p0Error> {
        self.fetch_all(sql, params, to_value_mapper).await
    }
}
