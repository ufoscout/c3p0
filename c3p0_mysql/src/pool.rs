use crate::error::into_c3p0_error;
use crate::mysql::driver::prelude::{FromValue, Queryable, ToValue};
use crate::mysql::driver::{Row, TxOpts};
use crate::mysql::r2d2::{MysqlConnectionManager, Pool, PooledConnection};
use c3p0_common::*;
use std::ops::DerefMut;

#[derive(Clone)]
pub struct MysqlC3p0Pool {
    pool: Pool<MysqlConnectionManager>,
}

impl MysqlC3p0Pool {
    pub fn new(pool: Pool<MysqlConnectionManager>) -> Self {
        MysqlC3p0Pool { pool }
    }
}

impl Into<MysqlC3p0Pool> for Pool<MysqlConnectionManager> {
    fn into(self) -> MysqlC3p0Pool {
        MysqlC3p0Pool::new(self)
    }
}

impl C3p0Pool for MysqlC3p0Pool {
    type CONN = MysqlConnection;

    fn transaction<T, E: From<C3p0Error>, F: FnOnce(&mut MysqlConnection) -> Result<T, E>>(
        &self,
        tx: F,
    ) -> Result<T, E> {
        let mut conn = self.pool.get().map_err(|err| C3p0Error::PoolError {
            cause: format!("{}", err),
        })?;

        let (result, executor) = {
            // ToDo: To avoid this unsafe we need GAT
            let transaction = unsafe {
                ::std::mem::transmute(
                    conn.start_transaction(TxOpts::default())
                        .map_err(into_c3p0_error)?,
                )
            };
            let mut sql_executor = MysqlConnection::Tx(transaction);
            let result = (tx)(&mut sql_executor)?;
            (result, sql_executor)
        };

        match executor {
            MysqlConnection::Tx(tx) => {
                tx.commit().map_err(into_c3p0_error)?;
            }
            _ => panic!("It should have been a transaction"),
        };

        Ok(result)
    }
}

pub enum MysqlConnection {
    Conn(PooledConnection<MysqlConnectionManager>),
    Tx(mysql_client::Transaction<'static>),
}

impl SqlConnection for MysqlConnection {
    fn batch_execute(&mut self, sql: &str) -> Result<(), C3p0Error> {
        match self {
            MysqlConnection::Conn(conn) => {
                let conn: &mut mysql_client::Conn = conn.deref_mut();
                batch_execute(conn, sql)
            }
            MysqlConnection::Tx(tx) => batch_execute(tx, sql),
        }
    }
}

impl MysqlConnection {
    pub fn execute(&mut self, sql: &str, params: &[&dyn ToValue]) -> Result<u64, C3p0Error> {
        match self {
            MysqlConnection::Conn(conn) => {
                let conn: &mut mysql_client::Conn = conn.deref_mut();
                execute(conn, sql, params)
            }
            MysqlConnection::Tx(tx) => execute(tx, sql, params),
        }
    }

    pub fn fetch_one_value<T: FromValue>(
        &mut self,
        sql: &str,
        params: &[&dyn ToValue],
    ) -> Result<T, C3p0Error> {
        match self {
            MysqlConnection::Conn(conn) => {
                let conn: &mut mysql_client::Conn = conn.deref_mut();
                fetch_one_value(conn, sql, params)
            }
            MysqlConnection::Tx(tx) => fetch_one_value(tx, sql, params),
        }
    }

    pub fn fetch_one<T, F: Fn(&Row) -> Result<T, Box<dyn std::error::Error>>>(
        &mut self,
        sql: &str,
        params: &[&dyn ToValue],
        mapper: F,
    ) -> Result<T, C3p0Error> {
        match self {
            MysqlConnection::Conn(conn) => {
                let conn: &mut mysql_client::Conn = conn.deref_mut();
                fetch_one(conn, sql, params, mapper)
            }
            MysqlConnection::Tx(tx) => fetch_one(tx, sql, params, mapper),
        }
    }

    pub fn fetch_one_optional<T, F: Fn(&Row) -> Result<T, Box<dyn std::error::Error>>>(
        &mut self,
        sql: &str,
        params: &[&dyn ToValue],
        mapper: F,
    ) -> Result<Option<T>, C3p0Error> {
        match self {
            MysqlConnection::Conn(conn) => {
                let conn: &mut mysql_client::Conn = conn.deref_mut();
                fetch_one_optional(conn, sql, params, mapper)
            }
            MysqlConnection::Tx(tx) => fetch_one_optional(tx, sql, params, mapper),
        }
    }

    pub fn fetch_all<T, F: Fn(&Row) -> Result<T, Box<dyn std::error::Error>>>(
        &mut self,
        sql: &str,
        params: &[&dyn ToValue],
        mapper: F,
    ) -> Result<Vec<T>, C3p0Error> {
        match self {
            MysqlConnection::Conn(conn) => {
                let conn: &mut mysql_client::Conn = conn.deref_mut();
                fetch_all(conn, sql, params, mapper)
            }
            MysqlConnection::Tx(tx) => fetch_all(tx, sql, params, mapper),
        }
    }

    pub fn fetch_all_values<T: FromValue>(
        &mut self,
        sql: &str,
        params: &[&dyn ToValue],
    ) -> Result<Vec<T>, C3p0Error> {
        match self {
            MysqlConnection::Conn(conn) => {
                let conn: &mut mysql_client::Conn = conn.deref_mut();
                fetch_all_values(conn, sql, params)
            }
            MysqlConnection::Tx(tx) => fetch_all_values(tx, sql, params),
        }
    }
}

fn execute<C: Queryable>(
    conn: &mut C,
    sql: &str,
    params: &[&dyn ToValue],
) -> Result<u64, C3p0Error> {
    conn.exec_iter(sql, params)
        .map(|row| row.affected_rows())
        .map_err(into_c3p0_error)
}

fn batch_execute<C: Queryable>(conn: &mut C, sql: &str) -> Result<(), C3p0Error> {
    conn.query_drop(sql).map_err(into_c3p0_error)
}

fn fetch_one_value<C: Queryable, T: FromValue>(
    conn: &mut C,
    sql: &str,
    params: &[&dyn ToValue],
) -> Result<T, C3p0Error> {
    fetch_one(conn, sql, params, to_value_mapper)
}

fn fetch_one<C: Queryable, T, F: Fn(&Row) -> Result<T, Box<dyn std::error::Error>>>(
    conn: &mut C,
    sql: &str,
    params: &[&dyn ToValue],
    mapper: F,
) -> Result<T, C3p0Error> {
    fetch_one_optional(conn, sql, params, mapper)
        .and_then(|result| result.ok_or_else(|| C3p0Error::ResultNotFoundError))
}

fn fetch_one_optional<C: Queryable, T, F: Fn(&Row) -> Result<T, Box<dyn std::error::Error>>>(
    conn: &mut C,
    sql: &str,
    params: &[&dyn ToValue],
    mapper: F,
) -> Result<Option<T>, C3p0Error> {
    conn.exec_iter(sql, params)
        .map_err(into_c3p0_error)?
        .next()
        .map(|result| {
            let row = result.map_err(into_c3p0_error)?;
            mapper(&row)
        })
        .transpose()
        .map_err(|err| C3p0Error::RowMapperError {
            cause: format!("{}", err),
        })
}

fn fetch_all<C: Queryable, T, F: Fn(&Row) -> Result<T, Box<dyn std::error::Error>>>(
    conn: &mut C,
    sql: &str,
    params: &[&dyn ToValue],
    mapper: F,
) -> Result<Vec<T>, C3p0Error> {
    conn.exec_iter(sql, params)
        .map_err(into_c3p0_error)?
        .map(|row| mapper(&row.map_err(into_c3p0_error)?))
        .collect::<Result<Vec<T>, Box<dyn std::error::Error>>>()
        .map_err(|err| C3p0Error::RowMapperError {
            cause: format!("{}", err),
        })
}

fn fetch_all_values<C: Queryable, T: FromValue>(
    conn: &mut C,
    sql: &str,
    params: &[&dyn ToValue],
) -> Result<Vec<T>, C3p0Error> {
    fetch_all(conn, sql, params, to_value_mapper)
}

fn to_value_mapper<T: FromValue>(row: &Row) -> Result<T, Box<dyn std::error::Error>> {
    let result = row
        .get_opt(0)
        .ok_or_else(|| C3p0Error::ResultNotFoundError)?;
    Ok(result.map_err(|err| C3p0Error::RowMapperError {
        cause: format!("{}", err),
    })?)
}
