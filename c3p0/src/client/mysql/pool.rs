use super::error::into_c3p0_error;
use crate::error::C3p0Error;
use crate::pool::{C3p0, Connection};
use mysql_client::prelude::GenericConnection;
use r2d2::{Pool, PooledConnection};
use r2d2_mysql::MysqlConnectionManager;
use std::cell::RefCell;
use std::ops::DerefMut;

pub type ToSql = mysql_client::Value;

pub struct C3p0MySqlBuilder {}

impl C3p0MySqlBuilder {
    pub fn build(pool: Pool<MysqlConnectionManager>) -> C3p0MySql {
        C3p0MySql { pool }
    }
}

pub struct C3p0MySql {
    pool: Pool<MysqlConnectionManager>,
}

impl C3p0 for C3p0MySql {
    type Connection = MySqlConnection;

    fn connection(&self) -> Result<Self::Connection, C3p0Error> {
        self.pool
            .get()
            .map_err(|err| C3p0Error::PoolError {
                cause: format!("{}", err),
            })
            .map(|conn| MySqlConnection {
                conn: RefCell::new(conn),
            })
    }

    fn transaction<T, F: Fn(&Connection) -> Result<T, C3p0Error>>(
        &self,
        tx: F,
    ) -> Result<T, C3p0Error> {
        let mut conn = self.pool.get().map_err(|err| C3p0Error::PoolError {
            cause: format!("{}", err),
        })?;

        let transaction = conn
            .start_transaction(true, None, None)
            .map_err(into_c3p0_error)?;
        let transaction = RefCell::new(transaction);
        let result = {
            let mut sql_executor = MySqlTransaction { tx: &transaction };
            (tx)(&mut sql_executor)?
        };
        transaction.into_inner().commit().map_err(into_c3p0_error)?;
        Ok(result)
    }
}

pub struct MySqlConnection {
    conn: RefCell<PooledConnection<MysqlConnectionManager>>,
}

impl Connection for MySqlConnection {
    fn execute(&self, sql: &str, params: &[&ToSql]) -> Result<u64, C3p0Error> {
        let mut conn_borrow = self.conn.borrow_mut();
        let conn: &mut mysql_client::Conn = conn_borrow.deref_mut();
        execute(conn, sql, params)
    }
}

pub struct MySqlTransaction<'a, T>
where
    T: GenericConnection,
{
    tx: &'a RefCell<T>,
}

impl<'a, T> Connection for MySqlTransaction<'a, T>
where
    T: GenericConnection,
{
    fn execute(&self, sql: &str, params: &[&ToSql]) -> Result<u64, C3p0Error> {
        let mut transaction = self.tx.borrow_mut();
        execute(transaction.deref_mut(), sql, params)
    }
}

fn execute<C: GenericConnection>(
    conn: &mut C,
    sql: &str,
    params: &[&ToSql],
) -> Result<u64, C3p0Error> {
    conn.prep_exec(sql, params.to_vec())
        .map(|row| row.affected_rows())
        .map_err(into_c3p0_error)
}
