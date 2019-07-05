pub mod error;

pub use c3p0_common::error::C3p0Error;

use crate::error::into_c3p0_error;
use crate::r2d2::{Pool, PooledConnection, SqliteConnectionManager};
use crate::rusqlite::types::{FromSql, ToSql};
use crate::rusqlite::Row;
use std::cell::RefCell;
use c3p0_common::pool::Connection;

pub mod r2d2 {
    pub use r2d2::*;
    pub use r2d2_sqlite::*;
}
pub mod rusqlite {
    pub use rusqlite::*;
}


pub struct C3p0SqliteBuilder {}

impl C3p0SqliteBuilder {
    pub fn build(pool: Pool<SqliteConnectionManager>) -> C3p0Sqlite {
        C3p0Sqlite { pool }
    }
}

#[derive(Clone)]
pub struct C3p0Sqlite {
    pool: Pool<SqliteConnectionManager>,
}

impl C3p0Sqlite {
    pub fn connection(&self) -> Result<SqliteConnectionOld, C3p0Error> {
        self.pool
            .get()
            .map_err(|err| C3p0Error::PoolError {
                cause: format!("{}", err),
            })
            .map(SqliteConnectionOld::Conn)
    }

    pub fn transaction<T, F: Fn(&SqliteConnectionOld) -> Result<T, Box<std::error::Error>>>(
        &self,
        tx: F,
    ) -> Result<T, C3p0Error> {
        let mut conn = self.pool.get().map_err(|err| C3p0Error::PoolError {
            cause: format!("{}", err),
        })?;

        let transaction = conn.transaction().map_err(into_c3p0_error)?;

        let result = {
            let mut sql_executor = SqliteConnectionOld::Tx(RefCell::new(transaction));
            let result = (tx)(&mut sql_executor)
                .map_err(|err| C3p0Error::TransactionError { cause: err })?;
            (result, sql_executor)
        };

        match result.1 {
            SqliteConnectionOld::Tx(tx) => {
                tx.into_inner().commit().map_err(into_c3p0_error)?;
            }
            _ => panic!("It should have been a transaction"),
        };

        Ok(result.0)
    }
}

fn new_simple_mut(conn: PooledConnection<SqliteConnectionManager>) -> Result<rentals::SimpleMut, C3p0Error> {
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


pub enum SqliteConnection<'a> {
    Conn(PooledConnection<SqliteConnectionManager>),
    Tx(RefCell<rusqlite::Transaction<'a>>),
}

impl<'a> Connection for SqliteConnection<'a> {
    fn batch_execute(&self, sql: &str) -> Result<(), C3p0Error> {
        match self {
            SqliteConnection::Conn(conn) => conn.execute_batch(sql).map_err(into_c3p0_error),
            SqliteConnection::Tx(tx) => tx.borrow_mut().execute_batch(sql).map_err(into_c3p0_error),
        }
    }
}

impl<'a> SqliteConnection<'a> {
    pub fn execute(&self, sql: &str, params: &[&ToSql]) -> Result<u64, C3p0Error> {
        match self {
            SqliteConnection::Conn(conn) => conn.execute(sql, params).map_err(into_c3p0_error).map(|res| res as u64),
            SqliteConnection::Tx(tx) => tx
                .borrow_mut()
                .execute(sql, params)
                .map(|res| res as u64)
                .map_err(into_c3p0_error),
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
            SqliteConnection::Tx(tx) => fetch_one_option(
                tx.borrow_mut().prepare(sql).map_err(into_c3p0_error)?,
                params,
                mapper,
            ),
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
            SqliteConnection::Tx(tx) => fetch_all(
                tx.borrow_mut().prepare(sql).map_err(into_c3p0_error)?,
                params,
                mapper,
            ),
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




pub enum SqliteConnectionOld<'a> {
    Conn(PooledConnection<SqliteConnectionManager>),
    Tx(RefCell<rusqlite::Transaction<'a>>),
}

impl<'a> Connection for SqliteConnectionOld<'a> {
    fn batch_execute(&self, sql: &str) -> Result<(), C3p0Error> {
        match self {
            SqliteConnectionOld::Conn(conn) => conn.execute_batch(sql).map_err(into_c3p0_error),
            SqliteConnectionOld::Tx(tx) => tx.borrow_mut().execute_batch(sql).map_err(into_c3p0_error),
        }
    }
}

impl<'a> SqliteConnectionOld<'a> {
    pub fn execute(&self, sql: &str, params: &[&ToSql]) -> Result<u64, C3p0Error> {
        match self {
            SqliteConnectionOld::Conn(conn) => conn.execute(sql, params).map_err(into_c3p0_error).map(|res| res as u64),
            SqliteConnectionOld::Tx(tx) => tx
                .borrow_mut()
                .execute(sql, params)
                .map(|res| res as u64)
                .map_err(into_c3p0_error),
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
            SqliteConnectionOld::Conn(conn) => {
                fetch_one_option(conn.prepare(sql).map_err(into_c3p0_error)?, params, mapper)
            }
            SqliteConnectionOld::Tx(tx) => fetch_one_option(
                tx.borrow_mut().prepare(sql).map_err(into_c3p0_error)?,
                params,
                mapper,
            ),
        }
    }

    pub fn fetch_all<T, F: Fn(&Row) -> Result<T, Box<std::error::Error>>>(
        &self,
        sql: &str,
        params: &[&ToSql],
        mapper: F,
    ) -> Result<Vec<T>, C3p0Error> {
        match self {
            SqliteConnectionOld::Conn(conn) => {
                fetch_all(conn.prepare(sql).map_err(into_c3p0_error)?, params, mapper)
            }
            SqliteConnectionOld::Tx(tx) => fetch_all(
                tx.borrow_mut().prepare(sql).map_err(into_c3p0_error)?,
                params,
                mapper,
            ),
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
