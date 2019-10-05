use crate::error::into_c3p0_error;
use crate::pg::driver::rows::Row;
use crate::pg::driver::types::{FromSql, ToSql};
use crate::pg::r2d2::{Pool, PooledConnection, PostgresConnectionManager};

use c3p0_common::*;

#[derive(Clone)]
pub struct C3p0PoolPg {
    pool: Pool<PostgresConnectionManager>,
}

impl C3p0PoolPg {
    pub fn new(pool: Pool<PostgresConnectionManager>) -> Self {
        C3p0PoolPg { pool }
    }
}

impl Into<C3p0PoolPg> for Pool<PostgresConnectionManager> {
    fn into(self) -> C3p0PoolPg {
        C3p0PoolPg::new(self)
    }
}

impl C3p0Pool for C3p0PoolPg {
    type CONN = PgConnection;

    fn connection(&self) -> Result<PgConnection, C3p0Error> {
        self.pool
            .get()
            .map_err(|err| C3p0Error::PoolError {
                cause: format!("{}", err),
            })
            .map(|conn| PgConnection { conn })
    }

    fn transaction<T, E: From<C3p0Error>, F: FnOnce(&PgConnection) -> Result<T, E>>(
        &self,
        tx: F,
    ) -> Result<T, E> {
        let conn = self.pool.get().map_err(|err| C3p0Error::PoolError {
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
    }
}

pub struct PgConnection {
    conn: PooledConnection<PostgresConnectionManager>,
}

impl SqlConnection for PgConnection {
    fn batch_execute(&self, sql: &str) -> Result<(), C3p0Error> {
        self.conn.batch_execute(sql).map_err(into_c3p0_error)
    }
}

impl PgConnection {
    pub fn execute(&self, sql: &str, params: &[&dyn ToSql]) -> Result<u64, C3p0Error> {
        self.conn.execute(sql, params).map_err(into_c3p0_error)
    }

    pub fn fetch_one_value<T: FromSql>(
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
        self.fetch_one_option(sql, params, mapper)
            .and_then(|result| result.ok_or_else(|| C3p0Error::ResultNotFoundError))
    }

    pub fn fetch_one_option<T, F: Fn(&Row) -> Result<T, Box<dyn std::error::Error>>>(
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

    pub fn fetch_all_values<T: FromSql>(
        &self,
        sql: &str,
        params: &[&dyn ToSql],
    ) -> Result<Vec<T>, C3p0Error> {
        self.fetch_all(sql, params, to_value_mapper)
    }
}

fn to_value_mapper<T: FromSql>(row: &Row) -> Result<T, Box<dyn std::error::Error>> {
    let result = row
        .get_opt(0)
        .ok_or_else(|| C3p0Error::ResultNotFoundError)?;
    Ok(result?)
}
