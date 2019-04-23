use super::error::into_c3p0_error;
use crate::error::C3p0Error;
use crate::pool::C3p0;
use mysql_client::prelude::FromValue as FromSql;
use mysql_client::{prelude::GenericConnection, prelude::ToValue};
use r2d2::{Pool, PooledConnection};
use r2d2_mysql::MysqlConnectionManager;
use std::cell::RefCell;
use std::ops::DerefMut;

pub type ToSql = ToValue;
pub type Row = mysql_client::Row;
pub type Connection = MySqlConnection;
pub type Transaction<'a, 'b> = MySqlTransaction<'a, mysql_client::Transaction<'b>>;

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
    fn connection(&self) -> Result<Connection, C3p0Error> {
        self.pool
            .get()
            .map_err(|err| C3p0Error::PoolError {
                cause: format!("{}", err),
            })
            .map(|conn| MySqlConnection {
                conn: RefCell::new(conn),
            })
    }

    fn transaction<T, F: Fn(&Transaction) -> Result<T, C3p0Error>>(
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

impl crate::pool::Connection for MySqlConnection {
    fn execute(&self, sql: &str, params: &[&ToSql]) -> Result<u64, C3p0Error> {
        let mut conn_borrow = self.conn.borrow_mut();
        let conn: &mut mysql_client::Conn = conn_borrow.deref_mut();
        execute(conn, sql, params)
    }

    fn batch_execute(&self, sql: &str) -> Result<(), C3p0Error> {
        let mut conn_borrow = self.conn.borrow_mut();
        let conn: &mut mysql_client::Conn = conn_borrow.deref_mut();
        batch_execute(conn, sql)
    }

    fn fetch_one_value<T: FromSql>(&self, sql: &str, params: &[&ToSql]) -> Result<T, C3p0Error> {
        let mut conn_borrow = self.conn.borrow_mut();
        let conn: &mut mysql_client::Conn = conn_borrow.deref_mut();
        fetch_one_value(conn, sql, params)
    }

    fn fetch_one<T, F: Fn(&Row) -> Result<T, C3p0Error>>(
        &self,
        sql: &str,
        params: &[&ToSql],
        mapper: F,
    ) -> Result<T, C3p0Error> {
        let mut conn_borrow = self.conn.borrow_mut();
        let conn: &mut mysql_client::Conn = conn_borrow.deref_mut();
        fetch_one(conn, sql, params, mapper)
    }

    fn fetch_one_option<T, F: Fn(&Row) -> Result<T, C3p0Error>>(
        &self,
        sql: &str,
        params: &[&ToSql],
        mapper: F,
    ) -> Result<Option<T>, C3p0Error> {
        let mut conn_borrow = self.conn.borrow_mut();
        let conn: &mut mysql_client::Conn = conn_borrow.deref_mut();
        fetch_one_option(conn, sql, params, mapper)
    }

    fn fetch_all<T, F: Fn(&Row) -> Result<T, C3p0Error>>(
        &self,
        sql: &str,
        params: &[&ToSql],
        mapper: F,
    ) -> Result<Vec<T>, C3p0Error> {
        let mut conn_borrow = self.conn.borrow_mut();
        let conn: &mut mysql_client::Conn = conn_borrow.deref_mut();
        fetch_all(conn, sql, params, mapper)
    }

    fn fetch_all_values<T: FromSql>(
        &self,
        sql: &str,
        params: &[&ToSql],
    ) -> Result<Vec<T>, C3p0Error> {
        let mut conn_borrow = self.conn.borrow_mut();
        let conn: &mut mysql_client::Conn = conn_borrow.deref_mut();
        fetch_all_values(conn, sql, params)
    }
}

pub struct MySqlTransaction<'a, C>
where
    C: GenericConnection,
{
    tx: &'a RefCell<C>,
}

impl<'t, C> crate::pool::Connection for MySqlTransaction<'t, C>
where
    C: GenericConnection,
{
    fn execute(&self, sql: &str, params: &[&ToSql]) -> Result<u64, C3p0Error> {
        let mut transaction = self.tx.borrow_mut();
        execute(transaction.deref_mut(), sql, params)
    }

    fn batch_execute(&self, sql: &str) -> Result<(), C3p0Error> {
        let mut transaction = self.tx.borrow_mut();
        batch_execute(transaction.deref_mut(), sql)
    }

    fn fetch_one_value<T: FromSql>(&self, sql: &str, params: &[&ToSql]) -> Result<T, C3p0Error> {
        let mut transaction = self.tx.borrow_mut();
        fetch_one_value(transaction.deref_mut(), sql, params)
    }

    fn fetch_one<T, F: Fn(&Row) -> Result<T, C3p0Error>>(
        &self,
        sql: &str,
        params: &[&ToSql],
        mapper: F,
    ) -> Result<T, C3p0Error> {
        let mut transaction = self.tx.borrow_mut();
        fetch_one(transaction.deref_mut(), sql, params, mapper)
    }

    fn fetch_one_option<T, F: Fn(&Row) -> Result<T, C3p0Error>>(
        &self,
        sql: &str,
        params: &[&ToSql],
        mapper: F,
    ) -> Result<Option<T>, C3p0Error> {
        let mut transaction = self.tx.borrow_mut();
        fetch_one_option(transaction.deref_mut(), sql, params, mapper)
    }

    fn fetch_all<T, F: Fn(&Row) -> Result<T, C3p0Error>>(
        &self,
        sql: &str,
        params: &[&ToSql],
        mapper: F,
    ) -> Result<Vec<T>, C3p0Error> {
        let mut transaction = self.tx.borrow_mut();
        fetch_all(transaction.deref_mut(), sql, params, mapper)
    }

    fn fetch_all_values<T: FromSql>(
        &self,
        sql: &str,
        params: &[&ToSql],
    ) -> Result<Vec<T>, C3p0Error> {
        let mut transaction = self.tx.borrow_mut();
        fetch_all_values(transaction.deref_mut(), sql, params)
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

fn fetch_one<C: GenericConnection, T, F: Fn(&Row) -> Result<T, C3p0Error>>(
    conn: &mut C,
    sql: &str,
    params: &[&ToSql],
    mapper: F,
) -> Result<T, C3p0Error> {
    fetch_one_option(conn, sql, params, mapper)
        .and_then(|result| result.ok_or_else(|| C3p0Error::ResultNotFoundError))
}

fn fetch_one_option<C: GenericConnection, T, F: Fn(&Row) -> Result<T, C3p0Error>>(
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
}

fn fetch_all<C: GenericConnection, T, F: Fn(&Row) -> Result<T, C3p0Error>>(
    conn: &mut C,
    sql: &str,
    params: &[&ToSql],
    mapper: F,
) -> Result<Vec<T>, C3p0Error> {
    conn.prep_exec(sql, params)
        .map_err(into_c3p0_error)?
        .map(|row| mapper(&row.map_err(into_c3p0_error)?))
        .collect()
}

fn fetch_all_values<C: GenericConnection, T: FromSql>(
    conn: &mut C,
    sql: &str,
    params: &[&ToSql],
) -> Result<Vec<T>, C3p0Error> {
    fetch_all(conn, sql, params, to_value_mapper)
}

fn to_value_mapper<T: FromSql>(row: &Row) -> Result<T, C3p0Error> {
    let result = row
        .get_opt(0)
        .ok_or_else(|| C3p0Error::ResultNotFoundError)?;
    Ok(result.map_err(|err| C3p0Error::SqlError {
        cause: format!("{}", err),
    })?)
}
