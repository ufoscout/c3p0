use async_trait::async_trait;
use c3p0_common::*;
use futures::Future;

use sqlx::postgres::PgPool;
use sqlx::{Transaction, PgConnection, Row, Type, Postgres, Connect};
use crate::into_c3p0_error;
use sqlx::pool::PoolConnection;
use sqlx::encode::Encode;

#[derive(Clone)]
pub struct PgC3p0PoolAsync {
    pool: PgPool,
}

impl PgC3p0PoolAsync {
    pub fn new(pool: PgPool) -> Self {
        PgC3p0PoolAsync { pool }
    }
}

impl Into<PgC3p0PoolAsync> for PgPool {
    fn into(self) -> PgC3p0PoolAsync {
        PgC3p0PoolAsync::new(self)
    }
}

#[async_trait]
impl C3p0PoolAsync for PgC3p0PoolAsync {
    type Conn = SqlxConnectionAsync<PgConnection>;

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

/*
impl PgConnectionAsync {
    pub async fn execute(
        &mut self,
        sql: &str,
        params: &[&(dyn Type<Postgres> + Encode<Postgres> + Sync)],
    ) -> Result<u64, C3p0Error> {
        match self {
            PgConnectionAsync::Tx(mut tx) =>{
                let mut query = sqlx::query(sql);
                for param in params {
                    query = query.bind(param);
                }
                query.execute(tx).await.map_err(into_c3p0_error)
            }
        }
    }

    /*
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
            .and_then(|result| result.ok_or_else(|| C3p0Error::ResultNotFoundError))
    }

    pub async fn fetch_one_optional<T, F: Fn(&Row) -> Result<T, Box<dyn std::error::Error>>>(
        &mut self,
        sql: &str,
        params: &[&(dyn ToSql + Sync)],
        mapper: F,
    ) -> Result<Option<T>, C3p0Error> {
        match self {
            PgConnectionAsync::Tx(tx) => {
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
            PgConnectionAsync::Tx(tx) => {
                let stmt = tx.prepare(sql).await.map_err(into_c3p0_error)?;
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

     */
}
*/