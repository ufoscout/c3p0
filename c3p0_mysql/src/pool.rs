use crate::error::into_c3p0_error;
use c3p0::error::C3p0Error;
use c3p0::pool::{C3p0, Connection};
use mysql::prelude::GenericConnection;
use r2d2::{Pool, PooledConnection};
use r2d2_mysql::MysqlConnectionManager;
use std::cell::RefCell;
use std::ops::DerefMut;
use mysql::Value;

pub struct MySqlC3p0 {
    pool: Pool<MysqlConnectionManager>,
}

impl C3p0 for MySqlC3p0 {
    type ToSql = Value;
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

    fn transaction<T, F: Fn(&Connection<Self::ToSql>) -> Result<T, C3p0Error>>(
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

impl Connection<Value> for MySqlConnection {
    fn execute(&self, sql: &str, params: &[&Value]) -> Result<u64, C3p0Error> {
        let mut conn_borrow = self.conn.borrow_mut();
        let conn: &mut mysql::Conn = conn_borrow.deref_mut();
        execute(conn, sql, params)
    }
}

pub struct MySqlTransaction<'a, T>
where
    T: GenericConnection,
{
    tx: &'a RefCell<T>,
}

impl<'a, T> Connection<Value> for MySqlTransaction<'a, T>
where
    T: GenericConnection,
{
    fn execute(&self, sql: &str, params: &[&Value]) -> Result<u64, C3p0Error> {
        let mut transaction = self.tx.borrow_mut();
        execute(transaction.deref_mut(), sql, params)
    }
}

fn execute<C: GenericConnection>(conn: &mut C, sql: &str, params: &[&Value]) -> Result<u64, C3p0Error> {
    conn.prep_exec(sql, params.to_vec())
        .map(|row| row.affected_rows())
        .map_err(into_c3p0_error)
}
