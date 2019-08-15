pub mod error;

pub use c3p0_common::error::C3p0Error;

use crate::error::into_c3p0_error;
use crate::r2d2::{Pool, PooledConnection, SqliteConnectionManager};
use crate::rusqlite::types::{FromSql, ToSql};
use crate::rusqlite::Row;
use std::cell::RefCell;

pub use c3p0_common::pool::{C3p0Pool, Connection};

pub mod r2d2 {
    pub use r2d2::*;
    pub use r2d2_sqlite::*;
}
pub mod rusqlite {
    pub use rusqlite::*;
}

pub mod json;

#[derive(Clone)]
pub struct C3p0PoolSqlite {
    pool: Pool<SqliteConnectionManager>,
}

impl C3p0PoolSqlite {
    pub fn new(pool: Pool<SqliteConnectionManager>) -> Self {
        C3p0PoolSqlite { pool }
    }
}

impl Into<C3p0PoolSqlite> for Pool<SqliteConnectionManager> {
    fn into(self) -> C3p0PoolSqlite {
        C3p0PoolSqlite::new(self)
    }
}

impl C3p0Pool for C3p0PoolSqlite {
    type CONN = SqliteConnection;

    fn connection(&self) -> Result<SqliteConnection, C3p0Error> {
        self.pool
            .get()
            .map_err(|err| C3p0Error::PoolError {
                cause: format!("{}", err),
            })
            .map(SqliteConnection::Conn)
    }

    fn transaction<T, F: FnOnce(&SqliteConnection) -> Result<T, Box<std::error::Error>>>(
        &self,
        tx: F,
    ) -> Result<T, C3p0Error> {
        let conn = self.pool.get().map_err(|err| C3p0Error::PoolError {
            cause: format!("{}", err),
        })?;

        let transaction = new_simple_mut(conn)?;

        let result = {
            let mut sql_executor = SqliteConnection::Tx(RefCell::new(transaction));
            let result = (tx)(&mut sql_executor)
                .map_err(|err| C3p0Error::TransactionError { cause: err })?;
            (result, sql_executor)
        };

        match result.1 {
            SqliteConnection::Tx(tx) => {
                let mut transaction = tx.borrow_mut();
                transaction
                    .rent_mut(|tref| {
                        let tref_some = tref.take();
                        tref_some.unwrap().commit()?;
                        Ok(())
                    })
                    .map_err(into_c3p0_error)?;
            }
            _ => panic!("It should have been a transaction"),
        };

        Ok(result.0)
    }
}

fn new_simple_mut(
    conn: PooledConnection<SqliteConnectionManager>,
) -> Result<rentals::SimpleMut, C3p0Error> {
    rentals::SimpleMut::try_new_or_drop(Box::new(conn), |c| {
        let tx = c.transaction().map_err(into_c3p0_error)?;
        Ok(Some(tx))
    })
}

#[macro_use]
extern crate rental;
rental! {
    mod rentals {
        use super::*;

        #[rental_mut]
        pub struct SimpleMut {
            conn: Box<PooledConnection<SqliteConnectionManager>>,
            tx: Option<rusqlite::Transaction<'conn>>,
        }
    }
}

pub enum SqliteConnection {
    Conn(PooledConnection<SqliteConnectionManager>),
    Tx(RefCell<rentals::SimpleMut>),
}

impl Connection for SqliteConnection {
    fn batch_execute(&self, sql: &str) -> Result<(), C3p0Error> {
        match self {
            SqliteConnection::Conn(conn) => conn.execute_batch(sql).map_err(into_c3p0_error),
            SqliteConnection::Tx(tx) => {
                let mut transaction = tx.borrow_mut();
                transaction.rent_mut(|tref| {
                    tref.as_mut()
                        .unwrap()
                        .execute_batch(sql)
                        .map_err(into_c3p0_error)
                })
                //tx.borrow_mut().execute_batch(sql).map_err(into_c3p0_error)
            }
        }
    }
}

impl SqliteConnection {
    pub fn execute(&self, sql: &str, params: &[&ToSql]) -> Result<u64, C3p0Error> {
        match self {
            SqliteConnection::Conn(conn) => conn
                .execute(sql, params)
                .map_err(into_c3p0_error)
                .map(|res| res as u64),
            SqliteConnection::Tx(tx) => {
                let mut transaction = tx.borrow_mut();
                transaction.rent_mut(|tref| {
                    tref.as_mut()
                        .unwrap()
                        .execute(sql, params)
                        .map(|res| res as u64)
                        .map_err(into_c3p0_error)
                })
            }
        }
    }

    pub fn fetch_one_value<T: FromSql>(
        &self,
        sql: &str,
        params: &[&ToSql],
    ) -> Result<T, C3p0Error> {
        self.fetch_one(sql, params, to_value_mapper)
    }

    pub fn fetch_one<T, F: Fn(&Row) -> Result<T, Box<std::error::Error>>>(
        &self,
        sql: &str,
        params: &[&ToSql],
        mapper: F,
    ) -> Result<T, C3p0Error> {
        self.fetch_one_option(sql, params, mapper)
            .and_then(|result| result.ok_or_else(|| C3p0Error::ResultNotFoundError))
    }

    pub fn fetch_one_option<T, F: Fn(&Row) -> Result<T, Box<std::error::Error>>>(
        &self,
        sql: &str,
        params: &[&ToSql],
        mapper: F,
    ) -> Result<Option<T>, C3p0Error> {
        match self {
            SqliteConnection::Conn(conn) => {
                fetch_one_option(conn.prepare(sql).map_err(into_c3p0_error)?, params, mapper)
            }
            SqliteConnection::Tx(tx) => {
                let mut transaction = tx.borrow_mut();
                transaction.rent_mut(|tref| {
                    fetch_one_option(
                        tref.as_mut()
                            .unwrap()
                            .prepare(sql)
                            .map_err(into_c3p0_error)?,
                        params,
                        mapper,
                    )
                })
            }
        }
    }

    pub fn fetch_all<T, F: Fn(&Row) -> Result<T, Box<std::error::Error>>>(
        &self,
        sql: &str,
        params: &[&ToSql],
        mapper: F,
    ) -> Result<Vec<T>, C3p0Error> {
        match self {
            SqliteConnection::Conn(conn) => {
                fetch_all(conn.prepare(sql).map_err(into_c3p0_error)?, params, mapper)
            }
            SqliteConnection::Tx(tx) => {
                let mut transaction = tx.borrow_mut();
                transaction.rent_mut(|tref| {
                    fetch_all(
                        tref.as_mut()
                            .unwrap()
                            .prepare(sql)
                            .map_err(into_c3p0_error)?,
                        params,
                        mapper,
                    )
                })
            }
        }
    }

    pub fn fetch_all_values<T: FromSql>(
        &self,
        sql: &str,
        params: &[&ToSql],
    ) -> Result<Vec<T>, C3p0Error> {
        self.fetch_all(sql, params, to_value_mapper)
    }
}

fn to_value_mapper<T: FromSql>(row: &Row) -> Result<T, Box<std::error::Error>> {
    let result = row.get(0).map_err(|err| C3p0Error::RowMapperError {
        cause: format!("{}", err),
    })?;
    Ok(result)
}

fn fetch_one_option<T, F: Fn(&Row) -> Result<T, Box<std::error::Error>>>(
    mut stmt: rusqlite::Statement,
    params: &[&ToSql],
    mapper: F,
) -> Result<Option<T>, C3p0Error> {
    let mut rows = stmt
        .query_and_then(params, |row| mapper(row))
        .map_err(|err| C3p0Error::RowMapperError {
            cause: format!("{}", err),
        })?;

    rows.next()
        .transpose()
        .map_err(|err| C3p0Error::RowMapperError {
            cause: format!("{}", err),
        })
}

fn fetch_all<T, F: Fn(&Row) -> Result<T, Box<std::error::Error>>>(
    mut stmt: rusqlite::Statement,
    params: &[&ToSql],
    mapper: F,
) -> Result<Vec<T>, C3p0Error> {
    let rows = stmt
        .query_and_then(params, |row| mapper(row))
        .map_err(|err| C3p0Error::RowMapperError {
            cause: format!("{}", err),
        })?;

    let mut result = vec![];
    for row in rows {
        result.push(row.map_err(|err| C3p0Error::RowMapperError {
            cause: format!("{}", err),
        })?)
    }

    Ok(result)
}
