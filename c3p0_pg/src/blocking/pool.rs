use crate::blocking::postgres::row::Row;
use crate::blocking::postgres::types::{FromSqlOwned, ToSql};
use crate::blocking::postgres::Transaction;
use crate::blocking::r2d2::{Pool, PooledConnection, PostgresConnectionManager};
use crate::blocking::*;
use c3p0_common::blocking::*;

#[derive(Clone)]
pub struct PgC3p0Pool {
    pool: Pool<PostgresConnectionManager>,
}

impl PgC3p0Pool {
    pub fn new(pool: Pool<PostgresConnectionManager>) -> Self {
        PgC3p0Pool { pool }
    }
}

impl Into<PgC3p0Pool> for Pool<PostgresConnectionManager> {
    fn into(self) -> PgC3p0Pool {
        PgC3p0Pool::new(self)
    }
}

impl C3p0Pool for PgC3p0Pool {
    type Conn = PgConnection;

    fn transaction<T, E: From<C3p0Error>, F: FnOnce(&mut PgConnection) -> Result<T, E>>(
        &self,
        tx: F,
    ) -> Result<T, E> {
        let mut conn = self.pool.get().map_err(|err| C3p0Error::PoolError {
            db: "postgres",
            pool: "r2d2",
            cause: format!("{}", err),
        })?;

        let (result, executor) = {
            // ToDo: To avoid this unsafe we need GAT
            let transaction = unsafe {
                ::std::mem::transmute(
                    conn.build_transaction()
                        .deferrable(true)
                        .start()
                        .map_err(into_c3p0_error)?,
                )
            };
            let mut sql_executor = PgConnection::Tx(transaction);
            let result = (tx)(&mut sql_executor);
            (result, sql_executor)
        };

        match executor {
            PgConnection::Tx(tx) => {
                if result.is_ok() {
                    tx.commit().map_err(into_c3p0_error)?;
                } else {
                    tx.rollback().map_err(into_c3p0_error)?;
                }
            }
            _ => panic!("It should have been a transaction"),
        };

        Ok(result?)
    }
}

pub enum PgConnection {
    Conn(PooledConnection<PostgresConnectionManager>),
    Tx(Transaction<'static>),
}

impl SqlConnection for PgConnection {
    fn batch_execute(&mut self, sql: &str) -> Result<(), C3p0Error> {
        match self {
            PgConnection::Conn(conn) => conn.batch_execute(sql).map_err(into_c3p0_error),
            PgConnection::Tx(tx) => tx.batch_execute(sql).map_err(into_c3p0_error),
        }
    }
}

impl PgConnection {
    pub fn execute(&mut self, sql: &str, params: &[&(dyn ToSql + Sync)]) -> Result<u64, C3p0Error> {
        match self {
            PgConnection::Conn(conn) => conn.execute(sql, params).map_err(into_c3p0_error),
            PgConnection::Tx(tx) => tx.execute(sql, params).map_err(into_c3p0_error),
        }
    }

    pub fn fetch_one_value<T: FromSqlOwned>(
        &mut self,
        sql: &str,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<T, C3p0Error> {
        self.fetch_one(sql, params, to_value_mapper)
    }

    pub fn fetch_one<T, F: Fn(&Row) -> Result<T, Box<dyn std::error::Error>>>(
        &mut self,
        sql: &str,
        params: &[&(dyn ToSql + Sync)],
        mapper: F,
    ) -> Result<T, C3p0Error> {
        self.fetch_one_optional(sql, params, mapper)
            .and_then(|result| result.ok_or_else(|| C3p0Error::ResultNotFoundError))
    }

    pub fn fetch_one_optional<T, F: Fn(&Row) -> Result<T, Box<dyn std::error::Error>>>(
        &mut self,
        sql: &str,
        params: &[&(dyn ToSql + Sync)],
        mapper: F,
    ) -> Result<Option<T>, C3p0Error> {
        match self {
            PgConnection::Conn(conn) => {
                let stmt = conn.prepare(sql).map_err(into_c3p0_error)?;
                conn.query(&stmt, params)
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
                let stmt = tx.prepare(sql).map_err(into_c3p0_error)?;
                tx.query(&stmt, params)
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

    pub fn fetch_all<T, F: Fn(&Row) -> Result<T, Box<dyn std::error::Error>>>(
        &mut self,
        sql: &str,
        params: &[&(dyn ToSql + Sync)],
        mapper: F,
    ) -> Result<Vec<T>, C3p0Error> {
        match self {
            PgConnection::Conn(conn) => {
                let stmt = conn.prepare(sql).map_err(into_c3p0_error)?;
                conn.query(&stmt, params)
                    .map_err(into_c3p0_error)?
                    .iter()
                    .map(|row| mapper(&row))
                    .collect::<Result<Vec<T>, Box<dyn std::error::Error>>>()
                    .map_err(|err| C3p0Error::RowMapperError {
                        cause: format!("{}", err),
                    })
            }
            PgConnection::Tx(tx) => {
                let stmt = tx.prepare(sql).map_err(into_c3p0_error)?;
                tx.query(&stmt, params)
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

    pub fn fetch_all_values<T: FromSqlOwned>(
        &mut self,
        sql: &str,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<Vec<T>, C3p0Error> {
        self.fetch_all(sql, params, to_value_mapper)
    }
}
