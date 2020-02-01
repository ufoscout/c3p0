use crate::error::{bb8_into_c3p0_error, into_c3p0_error};
use crate::pg::bb8::{Pool, PooledConnection, PostgresConnectionManager};
use crate::pg::driver::row::Row;
use crate::pg::driver::types::{FromSqlOwned, ToSql};

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

    async fn connection(&self) -> Result<Self::CONN, C3p0Error> {
        self.pool
            .get()
            .await
            .map_err(|err| C3p0Error::PoolError {
                cause: format!("{}", err),
            })
            .map(|conn| PgConnection::Conn(conn))
    }

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

fn to_value_mapper<T: FromSqlOwned>(row: &Row) -> Result<T, Box<dyn std::error::Error>> {
    Ok(row.try_get(0).map_err(|_| C3p0Error::ResultNotFoundError)?)
}
