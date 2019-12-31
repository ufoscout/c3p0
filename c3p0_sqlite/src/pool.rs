use crate::error::into_c3p0_error;
use crate::sqlite::driver::types::{FromSql, ToSql};
use crate::sqlite::driver::Row;
use crate::sqlite::r2d2::{Pool, PooledConnection, SqliteConnectionManager};

use c3p0_common::*;

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

    fn connection(&self) -> Result<SqliteConnection, C3p0Error> {
        self.pool
            .get()
            .map_err(|err| C3p0Error::PoolError {
                cause: format!("{}", err),
            })
            .map(SqliteConnection::Conn)
    }

    fn transaction<T, E: From<C3p0Error>, F: FnOnce(&mut SqliteConnection) -> Result<T, E>>(
        &self,
        tx: F,
    ) -> Result<T, E> {
        let conn = self.pool.get().map_err(|err| C3p0Error::PoolError {
            cause: format!("{}", err),
        })?;

        let transaction = new_simple_mut(conn)?;

        let (result, executor) = {
            let mut sql_executor = SqliteConnection::Tx(transaction);
            let result = (tx)(&mut sql_executor)?;
            (result, sql_executor)
        };

        match executor {
            SqliteConnection::Tx(mut tx) => {
                tx.rent_mut(|tref| {
                    let tref_some = tref.take();
                    tref_some.unwrap().commit()?;
                    Ok(())
                })
                .map_err(into_c3p0_error)?;
            }
            _ => panic!("It should have been a transaction"),
        };

        Ok(result)
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
    Tx(rentals::SimpleMut),
}

impl SqlConnection for SqliteConnection {
    fn batch_execute(&mut self, sql: &str) -> Result<(), C3p0Error> {
        match self {
            SqliteConnection::Conn(conn) => conn.execute_batch(sql).map_err(into_c3p0_error),
            SqliteConnection::Tx(tx) => {
                tx.rent_mut(|tref| {
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
    pub fn execute(&mut self, sql: &str, params: &[&dyn ToSql]) -> Result<u64, C3p0Error> {
        match self {
            SqliteConnection::Conn(conn) => conn
                .execute(sql, params)
                .map_err(into_c3p0_error)
                .map(|res| res as u64),
            SqliteConnection::Tx(tx) => tx.rent_mut(|tref| {
                tref.as_mut()
                    .unwrap()
                    .execute(sql, params)
                    .map(|res| res as u64)
                    .map_err(into_c3p0_error)
            }),
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
            SqliteConnection::Tx(tx) => tx.rent_mut(|tref| {
                fetch_one_optional(
                    tref.as_mut()
                        .unwrap()
                        .prepare(sql)
                        .map_err(into_c3p0_error)?,
                    params,
                    mapper,
                )
            }),
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
            SqliteConnection::Tx(tx) => tx.rent_mut(|tref| {
                fetch_all(
                    tref.as_mut()
                        .unwrap()
                        .prepare(sql)
                        .map_err(into_c3p0_error)?,
                    params,
                    mapper,
                )
            }),
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
