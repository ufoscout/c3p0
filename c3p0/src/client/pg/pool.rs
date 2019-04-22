use super::error::into_c3p0_error;
use crate::error::C3p0Error;
use crate::pool::{C3p0, Connection};
use r2d2::{Pool, PooledConnection};
use r2d2_postgres::PostgresConnectionManager;

pub type ToSql = postgres_shared::types::ToSql;

pub struct C3p0PgBuilder {}

impl C3p0PgBuilder {
    pub fn build(pool: Pool<PostgresConnectionManager>) -> C3p0Pg {
        C3p0Pg { pool }
    }
}

pub struct C3p0Pg {
    pool: Pool<PostgresConnectionManager>,
}

impl C3p0 for C3p0Pg {
    type Connection = PgConnection;

    fn connection(&self) -> Result<Self::Connection, C3p0Error> {
        self.pool
            .get()
            .map_err(|err| C3p0Error::PoolError {
                cause: format!("{}", err),
            })
            .map(|conn| PgConnection { conn })
    }

    fn transaction<T, F: Fn(&Connection) -> Result<T, C3p0Error>>(
        &self,
        tx: F,
    ) -> Result<T, C3p0Error> {
        let conn = self.pool.get().map_err(|err| C3p0Error::PoolError {
            cause: format!("{}", err),
        })?;
        let sql_executor = PgConnection { conn };

        let transaction = sql_executor.conn.transaction().map_err(into_c3p0_error)?;

        (tx)(&sql_executor).and_then(move |result| {
            transaction
                .commit()
                .map_err(into_c3p0_error)
                .map(|()| result)
        })
    }
}

pub struct PgConnection {
    conn: PooledConnection<PostgresConnectionManager>,
}

impl Connection for PgConnection {
    fn execute(&self, sql: &str, params: &[&ToSql]) -> Result<u64, C3p0Error> {
        self.conn.execute(sql, params).map_err(into_c3p0_error)
    }

    fn batch_execute(&self, sql: &str) -> Result<(), C3p0Error> {
        self.conn.batch_execute(sql).map_err(into_c3p0_error)
    }
}
