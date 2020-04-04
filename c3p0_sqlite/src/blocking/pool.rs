use crate::blocking::*;
use crate::blocking::rusqlite::types::{FromSql, ToSql};
use crate::blocking::rusqlite::Row;
use crate::blocking::r2d2::{Pool, PooledConnection, SqliteConnectionManager};

use c3p0_common::blocking::*;

#[derive(Clone)]
pub struct SqliteC3p0Pool {
    pool: Pool<SqliteConnectionManager>,
}

impl SqliteC3p0Pool {
    pub fn new(pool: Pool<SqliteConnectionManager>) -> Self {
        SqliteC3p0Pool { pool }
    }
}

impl Into<SqliteC3p0Pool> for Pool<SqliteConnectionManager> {
    fn into(self) -> SqliteC3p0Pool {
        SqliteC3p0Pool::new(self)
    }
}

impl C3p0Pool for SqliteC3p0Pool {
    type CONN = SqliteConnection;

    fn transaction<T, E: From<C3p0Error>, F: FnOnce(&mut SqliteConnection) -> Result<T, E>>(
        &self,
        tx: F,
    ) -> Result<T, E> {
        let mut conn = self.pool.get().map_err(|err| C3p0Error::PoolError {
            db: "sqlite",
            pool: "r2d2",
            cause: format!("{}", err),
        })?;

        let (result, executor) = {
            // ToDo: To avoid this unsafe we need GAT
            let transaction =
                unsafe { ::std::mem::transmute(conn.transaction().map_err(into_c3p0_error)?) };
            let mut sql_executor = SqliteConnection::Tx(transaction);
            let result = (tx)(&mut sql_executor)?;
            (result, sql_executor)
        };

        match executor {
            SqliteConnection::Tx(tx) => {
                tx.commit().map_err(into_c3p0_error)?;
            }
            _ => panic!("It should have been a transaction"),
        };

        Ok(result)
    }
}

pub enum SqliteConnection {
    Conn(PooledConnection<SqliteConnectionManager>),
    Tx(rusqlite::Transaction<'static>),
}

impl SqlConnection for SqliteConnection {
    fn batch_execute(&mut self, sql: &str) -> Result<(), C3p0Error> {
        match self {
            SqliteConnection::Conn(conn) => conn.execute_batch(sql).map_err(into_c3p0_error),
            SqliteConnection::Tx(tx) => tx.execute_batch(sql).map_err(into_c3p0_error),
        }
    }
}

impl SqliteConnection {
    pub fn execute(&mut self, sql: &str, params: &[&dyn ToSql]) -> Result<u64, C3p0Error> {
        match self {
            SqliteConnection::Conn(conn) => conn
                .execute(sql, params)
                .map_err(into_c3p0_error)
                .map(|res| res as u64),
            SqliteConnection::Tx(tx) => tx
                .execute(sql, params)
                .map(|res| res as u64)
                .map_err(into_c3p0_error),
        }
    }

    pub fn fetch_one_value<T: FromSql>(
        &mut self,
        sql: &str,
        params: &[&dyn ToSql],
    ) -> Result<T, C3p0Error> {
        self.fetch_one(sql, params, to_value_mapper)
    }

    pub fn fetch_one<T, F: Fn(&Row) -> Result<T, Box<dyn std::error::Error>>>(
        &mut self,
        sql: &str,
        params: &[&dyn ToSql],
        mapper: F,
    ) -> Result<T, C3p0Error> {
        self.fetch_one_optional(sql, params, mapper)
            .and_then(|result| result.ok_or_else(|| C3p0Error::ResultNotFoundError))
    }

    pub fn fetch_one_optional<T, F: Fn(&Row) -> Result<T, Box<dyn std::error::Error>>>(
        &mut self,
        sql: &str,
        params: &[&dyn ToSql],
        mapper: F,
    ) -> Result<Option<T>, C3p0Error> {
        match self {
            SqliteConnection::Conn(conn) => {
                fetch_one_optional(conn.prepare(sql).map_err(into_c3p0_error)?, params, mapper)
            }
            SqliteConnection::Tx(tx) => {
                fetch_one_optional(tx.prepare(sql).map_err(into_c3p0_error)?, params, mapper)
            }
        }
    }

    pub fn fetch_all<T, F: Fn(&Row) -> Result<T, Box<dyn std::error::Error>>>(
        &mut self,
        sql: &str,
        params: &[&dyn ToSql],
        mapper: F,
    ) -> Result<Vec<T>, C3p0Error> {
        match self {
            SqliteConnection::Conn(conn) => {
                fetch_all(conn.prepare(sql).map_err(into_c3p0_error)?, params, mapper)
            }
            SqliteConnection::Tx(tx) => {
                fetch_all(tx.prepare(sql).map_err(into_c3p0_error)?, params, mapper)
            }
        }
    }

    pub fn fetch_all_values<T: FromSql>(
        &mut self,
        sql: &str,
        params: &[&dyn ToSql],
    ) -> Result<Vec<T>, C3p0Error> {
        self.fetch_all(sql, params, to_value_mapper)
    }
}

fn to_value_mapper<T: FromSql>(row: &Row) -> Result<T, Box<dyn std::error::Error>> {
    let result = row.get(0).map_err(|err| C3p0Error::RowMapperError {
        cause: format!("{}", err),
    })?;
    Ok(result)
}

fn fetch_one_optional<T, F: Fn(&Row) -> Result<T, Box<dyn std::error::Error>>>(
    mut stmt: rusqlite::Statement,
    params: &[&dyn ToSql],
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

fn fetch_all<T, F: Fn(&Row) -> Result<T, Box<dyn std::error::Error>>>(
    mut stmt: rusqlite::Statement,
    params: &[&dyn ToSql],
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
