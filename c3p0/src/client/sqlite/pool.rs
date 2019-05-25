use super::error::into_c3p0_error;
use crate::error::C3p0Error;
use crate::pool::C3p0Base;
use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::types::FromSql;
use std::cell::RefCell;

pub type ToSql = rusqlite::types::ToSql;
pub type Row<'a> = rusqlite::Row<'a>;
pub type Connection<'a> = SqliteConnection<'a>;

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

impl C3p0Base for C3p0Sqlite {
    fn connection(&self) -> Result<Connection, C3p0Error> {
        self.pool
            .get()
            .map_err(|err| C3p0Error::PoolError {
                cause: format!("{}", err),
            })
            .map(|conn| SqliteConnection::Conn(conn))
    }

    fn transaction<T, F: Fn(&Connection) -> Result<T, Box<std::error::Error>>>(
        &self,
        tx: F,
    ) -> Result<T, C3p0Error> {
        let mut conn = self.pool.get().map_err(|err| C3p0Error::PoolError {
            cause: format!("{}", err),
        })?;

        let transaction = conn.transaction().map_err(into_c3p0_error)?;

        let result = {
            let mut sql_executor = SqliteConnection::Tx(RefCell::new(transaction));
            let result = (tx)(&mut sql_executor)
                .map_err(|err| C3p0Error::TransactionError { cause: err })?;
            (result, sql_executor)
        };

        match result.1 {
            SqliteConnection::Tx(tx) => {
                tx.into_inner().commit().map_err(into_c3p0_error)?;
            }
            _ => panic!("It should have been a transaction"),
        };

        Ok(result.0)
    }
}

pub enum SqliteConnection<'a> {
    Conn(PooledConnection<SqliteConnectionManager>),
    Tx(RefCell<rusqlite::Transaction<'a>>),
}

impl<'a> crate::pool::ConnectionBase for SqliteConnection<'a> {
    fn execute(&self, sql: &str, params: &[&ToSql]) -> Result<usize, C3p0Error> {
        match self {
            SqliteConnection::Conn(conn) => conn.execute(sql, params).map_err(into_c3p0_error),
            SqliteConnection::Tx(tx) => tx
                .borrow_mut()
                .execute(sql, params)
                .map_err(into_c3p0_error),
        }
    }

    fn batch_execute(&self, sql: &str) -> Result<(), C3p0Error> {
        match self {
            SqliteConnection::Conn(conn) => conn.execute_batch(sql).map_err(into_c3p0_error),
            SqliteConnection::Tx(tx) => tx.borrow_mut().execute_batch(sql).map_err(into_c3p0_error),
        }
    }

    fn fetch_one_value<T: FromSql>(&self, sql: &str, params: &[&ToSql]) -> Result<T, C3p0Error> {
        self.fetch_one(sql, params, to_value_mapper)
    }

    fn fetch_one<T, F: Fn(&Row) -> Result<T, Box<std::error::Error>>>(
        &self,
        sql: &str,
        params: &[&ToSql],
        mapper: F,
    ) -> Result<T, C3p0Error> {
        self.fetch_one_option(sql, params, mapper)
            .and_then(|result| result.ok_or_else(|| C3p0Error::ResultNotFoundError))
    }

    fn fetch_one_option<T, F: Fn(&Row) -> Result<T, Box<std::error::Error>>>(
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

    fn fetch_all<T, F: Fn(&Row) -> Result<T, Box<std::error::Error>>>(
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

    fn fetch_all_values<T: FromSql>(
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
        .query_and_then(params, |row| {
            mapper(row).map_err(|err| C3p0Error::RowMapperError {
                cause: format!("{}", err),
            })
        })
        .map_err(|err| C3p0Error::RowMapperError {
            cause: format!("{}", err),
        })?;

    rows.next().transpose()
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
