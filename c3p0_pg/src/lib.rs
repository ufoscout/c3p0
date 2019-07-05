pub mod error;

use crate::error::into_c3p0_error;
use crate::postgres::rows::Row;
use crate::postgres::types::{FromSql, ToSql};
use crate::r2d2::{Pool, PooledConnection, PostgresConnectionManager};

pub use c3p0_common::error::C3p0Error;
pub use c3p0_common::pool::{Connection, C3p0};

pub mod r2d2 {
    pub use r2d2::*;
    pub use r2d2_postgres::*;
}
pub mod postgres {
    pub use postgres::*;
}

pub struct C3p0PgBuilder {}

impl C3p0PgBuilder {
    pub fn build(pool: Pool<PostgresConnectionManager>) -> C3p0Pg {
        C3p0Pg { pool }
    }
}

#[derive(Clone)]
pub struct C3p0Pg {
    pool: Pool<PostgresConnectionManager>,
}

impl C3p0<PgConnection> for C3p0Pg {
    fn connection(&self) -> Result<PgConnection, C3p0Error> {
        self.pool
            .get()
            .map_err(|err| C3p0Error::PoolError {
                cause: format!("{}", err),
            })
            .map(|conn| PgConnection { conn })
    }

    fn transaction<T, F: Fn(&PgConnection) -> Result<T, Box<std::error::Error>>>(
        &self,
        tx: F,
    ) -> Result<T, C3p0Error> {
        let conn = self.pool.get().map_err(|err| C3p0Error::PoolError {
            cause: format!("{}", err),
        })?;
        let sql_executor = PgConnection { conn };

        let transaction = sql_executor.conn.transaction().map_err(into_c3p0_error)?;

        (tx)(&sql_executor)
            .map_err(|err| C3p0Error::TransactionError { cause: err })
            .and_then(move |result| {
                transaction
                    .commit()
                    .map_err(into_c3p0_error)
                    .map(|()| result)
            })
    }
}

pub struct PgConnection {
    conn: PooledConnection<PostgresConnectionManager>,
}

impl Connection for PgConnection {
    fn batch_execute(&self, sql: &str) -> Result<(), C3p0Error> {
        self.conn.batch_execute(sql).map_err(into_c3p0_error)
    }
}

impl PgConnection {
    pub fn execute(&self, sql: &str, params: &[&ToSql]) -> Result<u64, C3p0Error> {
        self.conn.execute(sql, params).map_err(into_c3p0_error)
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

    pub fn fetch_all<T, F: Fn(&Row) -> Result<T, Box<std::error::Error>>>(
        &self,
        sql: &str,
        params: &[&ToSql],
        mapper: F,
    ) -> Result<Vec<T>, C3p0Error> {
        let stmt = self.conn.prepare(sql).map_err(into_c3p0_error)?;
        stmt.query(params)
            .map_err(into_c3p0_error)?
            .iter()
            .map(|row| mapper(&row))
            .collect::<Result<Vec<T>, Box<std::error::Error>>>()
            .map_err(|err| C3p0Error::RowMapperError {
                cause: format!("{}", err),
            })
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
    let result = row
        .get_opt(0)
        .ok_or_else(|| C3p0Error::ResultNotFoundError)?;
    Ok(result?)
}
