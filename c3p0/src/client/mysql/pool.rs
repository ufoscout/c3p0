use super::error::into_c3p0_error;
use crate::error::C3p0Error;
use crate::pool::C3p0Base;
use mysql_client::prelude::FromValue as FromSql;
use mysql_client::{prelude::GenericConnection, prelude::ToValue};
use r2d2::{Pool, PooledConnection};
use r2d2_mysql::MysqlConnectionManager;
use std::cell::RefCell;
use std::ops::DerefMut;

pub type ToSql = ToValue;
pub type Row = mysql_client::Row;
pub type Connection<'a> = MySqlConnection<'a>;

pub struct C3p0MySqlBuilder {}

impl C3p0MySqlBuilder {
    pub fn build(pool: Pool<MysqlConnectionManager>) -> C3p0MySql {
        C3p0MySql { pool }
    }
}

#[derive(Clone)]
pub struct C3p0MySql {
    pool: Pool<MysqlConnectionManager>,
}

impl C3p0Base for C3p0MySql {
    fn connection(&self) -> Result<Connection, C3p0Error> {
        self.pool
            .get()
            .map_err(|err| C3p0Error::PoolError {
                cause: format!("{}", err),
            })
            .map(|conn| MySqlConnection::Conn(RefCell::new(conn)))
    }

    fn transaction<T, F: Fn(&Connection) -> Result<T, Box<std::error::Error>>>(
        &self,
        tx: F,
    ) -> Result<T, C3p0Error> {
        let mut conn = self.pool.get().map_err(|err| C3p0Error::PoolError {
            cause: format!("{}", err),
        })?;

        let transaction = conn
            .start_transaction(true, None, None)
            .map_err(into_c3p0_error)?;

        let result = {
            let mut sql_executor = MySqlConnection::Tx(RefCell::new(transaction));
            let result = (tx)(&mut sql_executor)
                .map_err(|err| C3p0Error::TransactionError { cause: err })?;
            (result, sql_executor)
        };

        match result.1 {
            MySqlConnection::Tx(tx) => {
                tx.into_inner().commit().map_err(into_c3p0_error)?;
            }
            _ => panic!("It should have been a transaction"),
        };

        Ok(result.0)
    }
}

pub enum MySqlConnection<'a> {
    Conn(RefCell<PooledConnection<MysqlConnectionManager>>),
    Tx(RefCell<mysql_client::Transaction<'a>>),
}

impl<'a> crate::pool::ConnectionBase for MySqlConnection<'a> {
    fn execute(&self, sql: &str, params: &[&ToSql]) -> Result<u64, C3p0Error> {
        match self {
            MySqlConnection::Conn(conn) => {
                let mut conn_borrow = conn.borrow_mut();
                let conn: &mut mysql_client::Conn = conn_borrow.deref_mut();
                execute(conn, sql, params)
            }
            MySqlConnection::Tx(tx) => {
                let mut transaction = tx.borrow_mut();
                execute(transaction.deref_mut(), sql, params)
            }
        }
    }

    fn batch_execute(&self, sql: &str) -> Result<(), C3p0Error> {
        match self {
            MySqlConnection::Conn(conn) => {
                let mut conn_borrow = conn.borrow_mut();
                let conn: &mut mysql_client::Conn = conn_borrow.deref_mut();
                batch_execute(conn, sql)
            }
            MySqlConnection::Tx(tx) => {
                let mut transaction = tx.borrow_mut();
                batch_execute(transaction.deref_mut(), sql)
            }
        }
    }

    fn fetch_one_value<T: FromSql>(&self, sql: &str, params: &[&ToSql]) -> Result<T, C3p0Error> {
        match self {
            MySqlConnection::Conn(conn) => {
                let mut conn_borrow = conn.borrow_mut();
                let conn: &mut mysql_client::Conn = conn_borrow.deref_mut();
                fetch_one_value(conn, sql, params)
            }
            MySqlConnection::Tx(tx) => {
                let mut transaction = tx.borrow_mut();
                fetch_one_value(transaction.deref_mut(), sql, params)
            }
        }
    }

    fn fetch_one<T, F: Fn(&Row) -> Result<T, Box<std::error::Error>>>(
        &self,
        sql: &str,
        params: &[&ToSql],
        mapper: F,
    ) -> Result<T, C3p0Error> {
        match self {
            MySqlConnection::Conn(conn) => {
                let mut conn_borrow = conn.borrow_mut();
                let conn: &mut mysql_client::Conn = conn_borrow.deref_mut();
                fetch_one(conn, sql, params, mapper)
            }
            MySqlConnection::Tx(tx) => {
                let mut transaction = tx.borrow_mut();
                fetch_one(transaction.deref_mut(), sql, params, mapper)
            }
        }
    }

    fn fetch_one_option<T, F: Fn(&Row) -> Result<T, Box<std::error::Error>>>(
        &self,
        sql: &str,
        params: &[&ToSql],
        mapper: F,
    ) -> Result<Option<T>, C3p0Error> {
        match self {
            MySqlConnection::Conn(conn) => {
                let mut conn_borrow = conn.borrow_mut();
                let conn: &mut mysql_client::Conn = conn_borrow.deref_mut();
                fetch_one_option(conn, sql, params, mapper)
            }
            MySqlConnection::Tx(tx) => {
                let mut transaction = tx.borrow_mut();
                fetch_one_option(transaction.deref_mut(), sql, params, mapper)
            }
        }
    }

    fn fetch_all<T, F: Fn(&Row) -> Result<T, Box<std::error::Error>>>(
        &self,
        sql: &str,
        params: &[&ToSql],
        mapper: F,
    ) -> Result<Vec<T>, C3p0Error> {
        match self {
            MySqlConnection::Conn(conn) => {
                let mut conn_borrow = conn.borrow_mut();
                let conn: &mut mysql_client::Conn = conn_borrow.deref_mut();
                fetch_all(conn, sql, params, mapper)
            }
            MySqlConnection::Tx(tx) => {
                let mut transaction = tx.borrow_mut();
                fetch_all(transaction.deref_mut(), sql, params, mapper)
            }
        }
    }

    fn fetch_all_values<T: FromSql>(
        &self,
        sql: &str,
        params: &[&ToSql],
    ) -> Result<Vec<T>, C3p0Error> {
        match self {
            MySqlConnection::Conn(conn) => {
                let mut conn_borrow = conn.borrow_mut();
                let conn: &mut mysql_client::Conn = conn_borrow.deref_mut();
                fetch_all_values(conn, sql, params)
            }
            MySqlConnection::Tx(tx) => {
                let mut transaction = tx.borrow_mut();
                fetch_all_values(transaction.deref_mut(), sql, params)
            }
        }
    }
}

fn execute<C: GenericConnection>(
    conn: &mut C,
    sql: &str,
    params: &[&ToSql],
) -> Result<u64, C3p0Error> {
    conn.prep_exec(sql, params)
        .map(|row| row.affected_rows())
        .map_err(into_c3p0_error)
}

fn batch_execute<C: GenericConnection>(conn: &mut C, sql: &str) -> Result<(), C3p0Error> {
    conn.query(sql).map(|_result| ()).map_err(into_c3p0_error)
}

fn fetch_one_value<C: GenericConnection, T: FromSql>(
    conn: &mut C,
    sql: &str,
    params: &[&ToSql],
) -> Result<T, C3p0Error> {
    fetch_one(conn, sql, params, to_value_mapper)
}

fn fetch_one<C: GenericConnection, T, F: Fn(&Row) -> Result<T, Box<std::error::Error>>>(
    conn: &mut C,
    sql: &str,
    params: &[&ToSql],
    mapper: F,
) -> Result<T, C3p0Error> {
    fetch_one_option(conn, sql, params, mapper)
        .and_then(|result| result.ok_or_else(|| C3p0Error::ResultNotFoundError))
}

fn fetch_one_option<C: GenericConnection, T, F: Fn(&Row) -> Result<T, Box<std::error::Error>>>(
    conn: &mut C,
    sql: &str,
    params: &[&ToSql],
    mapper: F,
) -> Result<Option<T>, C3p0Error> {
    conn.prep_exec(sql, params)
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

fn fetch_all<C: GenericConnection, T, F: Fn(&Row) -> Result<T, Box<std::error::Error>>>(
    conn: &mut C,
    sql: &str,
    params: &[&ToSql],
    mapper: F,
) -> Result<Vec<T>, C3p0Error> {
    conn.prep_exec(sql, params)
        .map_err(into_c3p0_error)?
        .map(|row| mapper(&row.map_err(into_c3p0_error)?))
        .collect::<Result<Vec<T>, Box<std::error::Error>>>()
        .map_err(|err| C3p0Error::RowMapperError {
            cause: format!("{}", err),
        })
}

fn fetch_all_values<C: GenericConnection, T: FromSql>(
    conn: &mut C,
    sql: &str,
    params: &[&ToSql],
) -> Result<Vec<T>, C3p0Error> {
    fetch_all(conn, sql, params, to_value_mapper)
}

fn to_value_mapper<T: FromSql>(row: &Row) -> Result<T, Box<std::error::Error>> {
    let result = row
        .get_opt(0)
        .ok_or_else(|| C3p0Error::ResultNotFoundError)?;
    Ok(result.map_err(|err| C3p0Error::SqlError {
        cause: format!("{}", err),
    })?)
}
