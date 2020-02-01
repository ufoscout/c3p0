use crate::error::{bb8_into_c3p0_error, into_c3p0_error};
use crate::pg_async::bb8::{Pool, PooledConnection, PostgresConnectionManager};
use crate::pg_async::driver::row::Row;
use crate::pg_async::driver::types::{FromSqlOwned, ToSql};

use async_trait::async_trait;
use c3p0_common::pool::{C3p0PoolAsync, SqlConnectionAsync};
use c3p0_common::*;
use futures::Future;
use tokio_postgres::{NoTls, Transaction};

#[derive(Clone)]
pub struct PgC3p0Pool {
    pool: Pool<PostgresConnectionManager<NoTls>>,
}

impl PgC3p0Pool {
    pub fn new(pool: Pool<PostgresConnectionManager<NoTls>>) -> Self {
        PgC3p0Pool { pool }
    }
}

impl Into<PgC3p0Pool> for Pool<PostgresConnectionManager<NoTls>> {
    fn into(self) -> PgC3p0Pool {
        PgC3p0Pool::new(self)
    }
}

#[async_trait]
impl<'a> C3p0PoolAsync for &'a PgC3p0Pool {
    type CONN = PgConnection<'a>;
/*
    async fn connection(&self) -> Result<Self::CONN, C3p0Error> {
        self.pool
            .get()
            .await
            .map_err(|err| C3p0Error::PoolError {
                cause: format!("{}", err),
            })
            .map(|conn| PgConnection::Conn(conn))
    }
*/
    async fn transaction<
        T: Send,
        E: From<C3p0Error>,
        F: Send + Sync + FnOnce(&mut Self::CONN) -> Fut,
        Fut: Send + Sync + Future<Output = Result<T, E>>,
    >(
        &self,
        tx: F,
    ) -> Result<T, E> {
        let mut conn = self.pool.get().await.map_err(bb8_into_c3p0_error)?;

        let (result, executor) = {
            // ToDo: To avoid this unsafe we need GAT
            let transaction = unsafe {
                ::std::mem::transmute(conn.transaction().await.map_err(into_c3p0_error)?)
            };
            let mut transaction = PgConnection::Tx(transaction);
            ((tx)(&mut transaction).await?, transaction)
        };

        match executor {
            PgConnection::Tx(tx) => {
                tx.commit().await.map_err(into_c3p0_error)?;
            }
            _ => panic!("It should have been a transaction"),
        };

        Ok(result)
    }
}

pub enum PgConnection<'a> {
    Conn(PooledConnection<'a, PostgresConnectionManager<NoTls>>),
    Tx(Transaction<'a>),
}

#[async_trait]
impl<'a> SqlConnectionAsync for &'a PgConnection<'a> {
    async fn batch_execute(&mut self, sql: &str) -> Result<(), C3p0Error> {
        match self {
            PgConnection::Conn(conn) => conn.batch_execute(sql).await.map_err(into_c3p0_error),
            PgConnection::Tx(tx) => tx.batch_execute(sql).await.map_err(into_c3p0_error),
        }
    }
}

impl <'a> PgConnection<'a> {
    pub async fn execute(&mut self, sql: &str, params: &[&(dyn ToSql + Sync)]) -> Result<u64, C3p0Error> {
        match self {
            PgConnection::Conn(conn) => conn.execute(sql, params).await.map_err(into_c3p0_error),
            PgConnection::Tx(tx) => tx.execute(sql, params).await.map_err(into_c3p0_error),
        }
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
        self.fetch_one_optional(sql, params, mapper).await
            .and_then(|result| result.ok_or_else(|| C3p0Error::ResultNotFoundError))
    }

    pub async fn fetch_one_optional<T, F: Fn(&Row) -> Result<T, Box<dyn std::error::Error>>>(
        &mut self,
        sql: &str,
        params: &[&(dyn ToSql + Sync)],
        mapper: F,
    ) -> Result<Option<T>, C3p0Error> {
        match self {
            PgConnection::Conn(conn) => {
                let stmt = conn.prepare(sql).await.map_err(into_c3p0_error)?;
                conn.query(&stmt, params)
                    .await
                    .map_err(into_c3p0_error)?
                    .iter()
                    .next()
                    .map(|row| mapper(&row))
                    .transpose()
                    .map_err(|err| C3p0Error::RowMapperError {
                        cause: format!("{}", err),
                    })
            }
            PgConnection::Tx(tx) => {
                let stmt = tx.prepare(sql).await.map_err(into_c3p0_error)?;
                tx.query(&stmt, params)
                    .await
                    .map_err(into_c3p0_error)?
                    .iter()
                    .next()
                    .map(|row| mapper(&row))
                    .transpose()
                    .map_err(|err| C3p0Error::RowMapperError {
                        cause: format!("{}", err),
                    })
            }
        }
    }

    pub async fn fetch_all<T, F: Fn(&Row) -> Result<T, Box<dyn std::error::Error>>>(
        &mut self,
        sql: &str,
        params: &[&(dyn ToSql + Sync)],
        mapper: F,
    ) -> Result<Vec<T>, C3p0Error> {
        match self {
            PgConnection::Conn(conn) => {
                let stmt = conn.prepare(sql).await.map_err(into_c3p0_error)?;
                conn.query(&stmt, params)
                    .await
                    .map_err(into_c3p0_error)?
                    .iter()
                    .map(|row| mapper(&row))
                    .collect::<Result<Vec<T>, Box<dyn std::error::Error>>>()
                    .map_err(|err| C3p0Error::RowMapperError {
                        cause: format!("{}", err),
                    })
            }
            PgConnection::Tx(tx) => {
                let stmt = tx.prepare(sql)
                    .await
                    .map_err(into_c3p0_error)?;
                tx.query(&stmt, params)
                    .await
                    .map_err(into_c3p0_error)?
                    .iter()
                    .map(|row| mapper(&row))
                    .collect::<Result<Vec<T>, Box<dyn std::error::Error>>>()
                    .map_err(|err| C3p0Error::RowMapperError {
                        cause: format!("{}", err),
                    })
            }
        }
    }

    pub async fn fetch_all_values<T: FromSqlOwned>(
        &mut self,
        sql: &str,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<Vec<T>, C3p0Error> {
        self.fetch_all(sql, params, to_value_mapper).await
    }
}

fn to_value_mapper<T: FromSqlOwned>(row: &Row) -> Result<T, Box<dyn std::error::Error>> {
    Ok(row.try_get(0).map_err(|_| C3p0Error::ResultNotFoundError)?)
}
