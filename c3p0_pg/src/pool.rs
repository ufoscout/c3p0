use c3p0::pool::{C3p0, Connection, SqlExecutor};
use c3p0::error::C3p0Error;
use r2d2::{Pool, PooledConnection};
use r2d2_postgres::PostgresConnectionManager;
use crate::error::into_c3p0_error;

pub struct C3p0Pg {
    pool: Pool<PostgresConnectionManager>
}

impl C3p0 for C3p0Pg {

    type Connection = PgConnection;

    fn connection(&self) -> Result<Self::Connection, C3p0Error> {
        self.pool.get().map_err(|err| C3p0Error::PoolError {cause: format!("{}", err)})
            .map(|conn| PgConnection{
                conn
            })
    }

}

pub struct PgConnection {
    conn: PooledConnection<PostgresConnectionManager>
}

impl Connection for PgConnection {

    fn transaction<T, F: Fn(&SqlExecutor) -> Result<T, C3p0Error>>(&self, tx: F) -> Result<T, C3p0Error> {
        let transaction = self.conn.transaction().map_err(into_c3p0_error)?;
        let mut sql_executor = PgSqlExecutor{ conn: &self.conn };
        (tx)(&mut sql_executor).and_then(move |result| transaction.commit().map_err(into_c3p0_error).map(|()| result))
    }

}

impl SqlExecutor for PgConnection {
    fn execute(&self, sql: &str) -> Result<u64, C3p0Error> {
        execute(&self.conn, sql)
    }
}

pub struct PgSqlExecutor<'a> {
    conn: &'a PooledConnection<PostgresConnectionManager>
}
impl <'a> SqlExecutor for PgSqlExecutor<'a> {
    fn execute(&self, sql: &str) -> Result<u64, C3p0Error> {
        execute(self.conn, sql)
    }
}

fn execute(conn: &PooledConnection<PostgresConnectionManager>, sql: &str) -> Result<u64, C3p0Error> {
    conn.execute(sql, &[]).map_err(into_c3p0_error)
}