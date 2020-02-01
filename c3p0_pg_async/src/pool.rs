use crate::error::{into_c3p0_error, bb8_into_c3p0_error};
use crate::pg::driver::row::Row;
use crate::pg::driver::types::{FromSqlOwned, ToSql};
use crate::pg::bb8::{Pool, PooledConnection, PostgresConnectionManager};

use async_trait::async_trait;
use c3p0_common::*;
use tokio_postgres::{NoTls, Client, Transaction};
use c3p0_common::pool::C3p0PoolAsync;
use futures::Future;
use std::ops::{Deref, DerefMut};

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
impl <'b> C3p0PoolAsync for &'b PgC3p0Pool {
    type CONN = PgConnection<'b>;

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

    async fn transaction<T, E: From<C3p0Error>, F: Send + Sync + FnOnce(&PgConnection<'b>) -> Fut, Fut: Send + Sync + Future<Output = Result<T, E>>>(
        &self,
        tx: F,
    ) -> Result<T, E> {

        let mut conn = self.pool.get().await.map_err(bb8_into_c3p0_error)?;
        let transaction = conn.transaction().await.map_err(into_c3p0_error)?;
        let mut transaction = PgConnection::Tx(transaction);

        (tx)(&mut transaction).await?;

        /*
        let (result, executor) = {
            let mut sql_executor = PgConnection::Tx(transaction);
            let result = (tx)(&mut sql_executor)?;
            (result, sql_executor)
        };

        match executor {
            PgConnection::Tx(mut tx) => {
                tx.rent_mut(|tref| {
                    let tref_some = tref.take();
                    tref_some.unwrap().commit()?;
                    Ok(())
                })
                    .map_err(into_c3p0_error)?;
            }
            _ => panic!("It should have been a transaction"),
        };

        Ok(result);
            */
        unimplemented!()
        /*
        let conn = self.pool.get().await.map_err(|err| C3p0Error::PoolError {
            cause: format!("{}", err),
        })?;
        let sql_executor = PgConnection { conn };

        let transaction = sql_executor.conn.transaction().map_err(into_c3p0_error)?;

        (tx)(&sql_executor).and_then(move |result| {
            Ok(transaction
                .commit()
                .map_err(into_c3p0_error)
                .map(|()| result)?)
        })
        */
    }
}


impl PgC3p0Pool {

    async fn transaction<T, E, F: Send + Sync + FnOnce(&Transaction) -> Fut, Fut: Send + Sync + Future<Output=Result<T, E>>>(
        &self,
        tx: F,
    ) -> Result<T, E> {
        let mut conn = self.pool.get().await.unwrap();
        let mut transaction = conn.transaction().await.unwrap();

        (tx)(&mut transaction).await?;

        /*
        let (result, executor) = {
            let mut sql_executor = PgConnection::Tx(transaction);
            let result = (tx)(&mut sql_executor)?;
            (result, sql_executor)
        };

        match executor {
            PgConnection::Tx(mut tx) => {
                tx.rent_mut(|tref| {
                    let tref_some = tref.take();
                    tref_some.unwrap().commit()?;
                    Ok(())
                })
                    .map_err(into_c3p0_error)?;
            }
            _ => panic!("It should have been a transaction"),
        };

        Ok(result);
            */
        unimplemented!()
        /*
        let conn = self.pool.get().await.map_err(|err| C3p0Error::PoolError {
            cause: format!("{}", err),
        })?;
        let sql_executor = PgConnection { conn };

        let transaction = sql_executor.conn.transaction().map_err(into_c3p0_error)?;

        (tx)(&sql_executor).and_then(move |result| {
            Ok(transaction
                .commit()
                .map_err(into_c3p0_error)
                .map(|()| result)?)
        })
        */
    }
}


pub enum PgConnection<'a> {
    Conn(PooledConnection<'a, PostgresConnectionManager<NoTls>>),
    Tx(Transaction<'a>),
}

/*
impl PgConnection {
    pub fn execute(&self, sql: &str, params: &[&dyn ToSql]) -> Result<u64, C3p0Error> {
        self.conn.execute(sql, params).map_err(into_c3p0_error)
    }

    pub fn fetch_one_value<T: FromSqlOwned>(
        &self,
        sql: &str,
        params: &[&dyn ToSql],
    ) -> Result<T, C3p0Error> {
        self.fetch_one(sql, params, to_value_mapper)
    }

    pub fn fetch_one<T, F: Fn(&Row) -> Result<T, Box<dyn std::error::Error>>>(
        &self,
        sql: &str,
        params: &[&dyn ToSql],
        mapper: F,
    ) -> Result<T, C3p0Error> {
        self.fetch_one_optional(sql, params, mapper)
            .and_then(|result| result.ok_or_else(|| C3p0Error::ResultNotFoundError))
    }

    pub fn fetch_one_optional<T, F: Fn(&Row) -> Result<T, Box<dyn std::error::Error>>>(
        &self,
        sql: &str,
        params: &[&dyn ToSql],
        mapper: F,
    ) -> Result<Option<T>, C3p0Error> {
        let stmt = self.conn.prepare(sql).map_err(into_c3p0_error)?;
        stmt.query(params)
            .map_err(into_c3p0_error)?
            .iter()
            .next()
            .map(|row| mapper(&row))
            .transpose()
            .map_err(|err| C3p0Error::RowMapperError {
                cause: format!("{}", err),
            })
    }

    pub fn fetch_all<T, F: Fn(&Row) -> Result<T, Box<dyn std::error::Error>>>(
        &self,
        sql: &str,
        params: &[&dyn ToSql],
        mapper: F,
    ) -> Result<Vec<T>, C3p0Error> {
        let stmt = self.conn.prepare(sql).map_err(into_c3p0_error)?;
        stmt.query(params)
            .map_err(into_c3p0_error)?
            .iter()
            .map(|row| mapper(&row))
            .collect::<Result<Vec<T>, Box<dyn std::error::Error>>>()
            .map_err(|err| C3p0Error::RowMapperError {
                cause: format!("{}", err),
            })
    }

    pub fn fetch_all_values<T: FromSqlOwned>(
        &self,
        sql: &str,
        params: &[&dyn ToSql],
    ) -> Result<Vec<T>, C3p0Error> {
        self.fetch_all(sql, params, to_value_mapper)
    }
}
*/

/*
fn to_value_mapper<T: FromSqlOwned>(row: &Row) -> Result<T, Box<dyn std::error::Error>> {
    let result = row
        .get_opt(0)
        .ok_or_else(|| C3p0Error::ResultNotFoundError)?;
    Ok(result?)
}
*/