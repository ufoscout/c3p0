use c3p0::pool::{C3p0, Connection, SqlExecutor};
use c3p0::error::C3p0Error;
use r2d2_mysql::MysqlConnectionManager;
use r2d2::{Pool, PooledConnection};
use crate::error::into_c3p0_error;
use mysql::prelude::GenericConnection;
use std::ops::DerefMut;

pub struct MySqlC3p0 {
    pool: Pool<MysqlConnectionManager>
}

impl C3p0 for MySqlC3p0 {

    type Connection = MySqlConnection;

    fn connection(&self) -> Result<Self::Connection, C3p0Error> {
        self.pool.get().map_err(|err| C3p0Error::PoolError {cause: format!("{}", err)})
            .map(|conn| MySqlConnection{
                conn
            })
    }

}

pub struct MySqlConnection {
    conn: PooledConnection<MysqlConnectionManager>
}

impl Connection for MySqlConnection {

    fn transaction<T, F: Fn(&mut SqlExecutor) -> Result<T, C3p0Error>>(&mut self, tx: F) -> Result<T, C3p0Error> {
        let mut transaction = self.conn.start_transaction(true, None, None).map_err(into_c3p0_error)?;
        let mut sql_executor = MySqlExecutor{ tx: &mut transaction };
        (tx)(&mut sql_executor).and_then(move |result| transaction.commit().map_err(into_c3p0_error).map(|()| result))
    }

}

impl SqlExecutor for MySqlConnection {
    fn execute(&mut self, sql: &str) -> Result<u64, C3p0Error> {
        let conn: &mut mysql::Conn = self.conn.deref_mut();
        execute(conn, sql)
    }
}

pub struct MySqlExecutor<'a, T>
where T: GenericConnection
{
    tx: &'a mut T
}

impl <'a, T> SqlExecutor for MySqlExecutor<'a, T>
    where T: GenericConnection {
    fn execute(&mut self, sql: &str) -> Result<u64, C3p0Error> {
        execute(self.tx, sql)
    }
}

fn execute<C: GenericConnection>(conn: &mut C, sql: &str) -> Result<u64, C3p0Error> {
    conn.prep_exec(sql, ())
        .map(|row| row.affected_rows())
        .map_err(into_c3p0_error)
}