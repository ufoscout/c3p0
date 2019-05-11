use super::error::into_c3p0_error;
use crate::error::C3p0Error;
use crate::pool::C3p0Base;
use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::types::FromSql;

pub type ToSql = rusqlite::types::ToSql;
pub type Row<'a> = rusqlite::Row<'a>;
pub type Connection = SqliteConnection;

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
            .map(|conn| SqliteConnection { conn })
    }

    fn transaction<T, F: Fn(&Connection) -> Result<T, Box<std::error::Error>>>(
        &self,
        tx: F,
    ) -> Result<T, C3p0Error> {
        let mut conn = self.pool.get().map_err(|err| C3p0Error::PoolError {
            cause: format!("{}", err),
        })?;
        let mut sql_executor = SqliteConnection { conn };

        let transaction = sql_executor.conn.transaction().map_err(into_c3p0_error)?;

        transaction.execute_batch()(tx)(&sql_executor)
            .map_err(|err| C3p0Error::TransactionError { cause: err })
            .and_then(move |result| {
                transaction
                    .commit()
                    .map_err(into_c3p0_error)
                    .map(|()| result)
            })
    }
}

pub struct SqliteConnection {
    conn: PooledConnection<SqliteConnectionManager>,
}

impl crate::pool::ConnectionBase for SqliteConnection {
    fn execute(&self, sql: &str, params: &[&ToSql]) -> Result<usize, C3p0Error> {
        self.conn.execute(sql, params).map_err(into_c3p0_error)
    }

    fn batch_execute(&self, sql: &str) -> Result<(), C3p0Error> {
        self.conn.execute_batch(sql).map_err(into_c3p0_error)
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
        let mut stmt = self.conn.prepare(sql).map_err(into_c3p0_error)?;

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
        &self,
        sql: &str,
        params: &[&ToSql],
        mapper: F,
    ) -> Result<Vec<T>, C3p0Error> {
        let mut stmt = self.conn.prepare(sql).map_err(into_c3p0_error)?;
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
