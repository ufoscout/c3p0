pub mod error;

pub use c3p0_common::error::C3p0Error;

use crate::error::into_c3p0_error;
use crate::mysql::prelude::{FromValue, GenericConnection, ToValue};
use crate::mysql::Row;
use crate::r2d2::{Pool, PooledConnection, MysqlConnectionManager};
use std::cell::RefCell;
use std::ops::DerefMut;
use c3p0_common::pool::Connection;

pub mod r2d2 {
    pub use r2d2::*;
    pub use r2d2_mysql::*;
}
pub mod mysql {
    pub use mysql_client::*;
}

pub struct C3p0MysqlBuilder {}

impl C3p0MysqlBuilder {
    pub fn build(pool: Pool<MysqlConnectionManager>) -> C3p0Mysql {
        C3p0Mysql { pool }
    }
}

#[derive(Clone)]
pub struct C3p0Mysql {
    pool: Pool<MysqlConnectionManager>,
}

impl C3p0Mysql {
    pub fn connection(&self) -> Result<MySqlConnection, C3p0Error> {
        self.pool
            .get()
            .map_err(|err| C3p0Error::PoolError {
                cause: format!("{}", err),
            })
            .map(|conn| MySqlConnection::Conn(RefCell::new(conn)))
    }

    pub fn transaction<T, F: Fn(&MySqlConnection) -> Result<T, Box<std::error::Error>>>(
        &self,
        tx: F,
    ) -> Result<T, C3p0Error> {
        let conn = self.pool.get().map_err(|err| C3p0Error::PoolError {
            cause: format!("{}", err),
        })?;

        let rent = new_simple_mut(conn)?;

        let result = {


            let mut sql_executor = MySqlConnection::Tx(RefCell::new(rent));
            let result = (tx)(&mut sql_executor)
                .map_err(|err| C3p0Error::TransactionError { cause: err })?;
            (result, sql_executor)
        };

        match result.1 {
            MySqlConnection::Tx(tx) => {
                let mut transaction = tx.borrow_mut();
                transaction.rent_mut(|tref| {
                    let tref_some = tref.take();
                    tref_some.unwrap().commit()?;
                    Ok(())
                }).map_err(into_c3p0_error)?;
            }
            _ => panic!("It should have been a transaction"),
        };

        Ok(result.0)
    }
}

fn new_simple_mut(conn: PooledConnection<MysqlConnectionManager>) -> Result<rentals::SimpleMut, C3p0Error> {
    rentals::SimpleMut::try_new_or_drop(Box::new(conn), |c| {
        let tx = c.start_transaction(true, None, None).map_err(into_c3p0_error)?;
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
			conn: Box<PooledConnection<MysqlConnectionManager>>,
			tx: Option<mysql_client::Transaction<'conn>>,
		}
	}
}

pub enum MySqlConnection {
    Conn(RefCell<PooledConnection<MysqlConnectionManager>>),
    Tx(RefCell<rentals::SimpleMut>),
}


impl Connection for MySqlConnection {
    fn batch_execute(&self, sql: &str) -> Result<(), C3p0Error> {
        match self {
            MySqlConnection::Conn(conn) => {
                let mut conn_borrow = conn.borrow_mut();
                let conn: &mut mysql_client::Conn = conn_borrow.deref_mut();
                batch_execute(conn, sql)
            }
            MySqlConnection::Tx(tx) => {
                let mut transaction = tx.borrow_mut();
                transaction.rent_mut(|tref| {
                    batch_execute(tref.as_mut().unwrap(), sql)
                })
            }
        }
    }
}

impl MySqlConnection {
    pub fn execute(&self, sql: &str, params: &[&ToValue]) -> Result<u64, C3p0Error> {
        match self {
            MySqlConnection::Conn(conn) => {
                let mut conn_borrow = conn.borrow_mut();
                let conn: &mut mysql_client::Conn = conn_borrow.deref_mut();
                execute(conn, sql, params)
            }
            MySqlConnection::Tx(tx) => {
                let mut transaction = tx.borrow_mut();
                transaction.rent_mut(|tref| {
                    execute(tref.as_mut().unwrap(), sql, params)
                })
            }
        }
    }

    pub fn fetch_one_value<T: FromValue>(
        &self,
        sql: &str,
        params: &[&ToValue],
    ) -> Result<T, C3p0Error> {
        match self {
            MySqlConnection::Conn(conn) => {
                let mut conn_borrow = conn.borrow_mut();
                let conn: &mut mysql_client::Conn = conn_borrow.deref_mut();
                fetch_one_value(conn, sql, params)
            }
            MySqlConnection::Tx(tx) => {
                let mut transaction = tx.borrow_mut();
                transaction.rent_mut(|tref| {
                    fetch_one_value(tref.as_mut().unwrap(), sql, params)
                })
            }
        }
    }

    pub fn fetch_one<T, F: Fn(&Row) -> Result<T, Box<std::error::Error>>>(
        &self,
        sql: &str,
        params: &[&ToValue],
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
                transaction.rent_mut(|tref| {
                    fetch_one(tref.as_mut().unwrap(), sql, params, mapper)
                })
            }
        }
    }

    pub fn fetch_one_option<T, F: Fn(&Row) -> Result<T, Box<std::error::Error>>>(
        &self,
        sql: &str,
        params: &[&ToValue],
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
                transaction.rent_mut(|tref| {
                    fetch_one_option(tref.as_mut().unwrap(), sql, params, mapper)
                })
            }
        }
    }

    pub fn fetch_all<T, F: Fn(&Row) -> Result<T, Box<std::error::Error>>>(
        &self,
        sql: &str,
        params: &[&ToValue],
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
                transaction.rent_mut(|tref| {
                    fetch_all(tref.as_mut().unwrap(), sql, params, mapper)
                })
            }
        }
    }

    pub fn fetch_all_values<T: FromValue>(
        &self,
        sql: &str,
        params: &[&ToValue],
    ) -> Result<Vec<T>, C3p0Error> {
        match self {
            MySqlConnection::Conn(conn) => {
                let mut conn_borrow = conn.borrow_mut();
                let conn: &mut mysql_client::Conn = conn_borrow.deref_mut();
                fetch_all_values(conn, sql, params)
            }
            MySqlConnection::Tx(tx) => {
                let mut transaction = tx.borrow_mut();
                transaction.rent_mut(|tref| {
                    fetch_all_values(tref.as_mut().unwrap(), sql, params)
                })
            }
        }
    }
}

fn execute<C: GenericConnection>(
    conn: &mut C,
    sql: &str,
    params: &[&ToValue],
) -> Result<u64, C3p0Error> {
    conn.prep_exec(sql, params)
        .map(|row| row.affected_rows())
        .map_err(into_c3p0_error)
}

fn batch_execute<C: GenericConnection>(conn: &mut C, sql: &str) -> Result<(), C3p0Error> {
    conn.query(sql).map(|_result| ()).map_err(into_c3p0_error)
}

fn fetch_one_value<C: GenericConnection, T: FromValue>(
    conn: &mut C,
    sql: &str,
    params: &[&ToValue],
) -> Result<T, C3p0Error> {
    fetch_one(conn, sql, params, to_value_mapper)
}

fn fetch_one<C: GenericConnection, T, F: Fn(&Row) -> Result<T, Box<std::error::Error>>>(
    conn: &mut C,
    sql: &str,
    params: &[&ToValue],
    mapper: F,
) -> Result<T, C3p0Error> {
    fetch_one_option(conn, sql, params, mapper)
        .and_then(|result| result.ok_or_else(|| C3p0Error::ResultNotFoundError))
}

fn fetch_one_option<C: GenericConnection, T, F: Fn(&Row) -> Result<T, Box<std::error::Error>>>(
    conn: &mut C,
    sql: &str,
    params: &[&ToValue],
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
    params: &[&ToValue],
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

fn fetch_all_values<C: GenericConnection, T: FromValue>(
    conn: &mut C,
    sql: &str,
    params: &[&ToValue],
) -> Result<Vec<T>, C3p0Error> {
    fetch_all(conn, sql, params, to_value_mapper)
}

fn to_value_mapper<T: FromValue>(row: &Row) -> Result<T, Box<std::error::Error>> {
    let result = row
        .get_opt(0)
        .ok_or_else(|| C3p0Error::ResultNotFoundError)?;
    Ok(result.map_err(|err| C3p0Error::SqlError {
        cause: format!("{}", err),
    })?)
}
