#![cfg(feature = "sqlx_sqlite")]

use c3p0::sqlx::sqlx::sqlite::*;
use c3p0::sqlx::*;
pub use c3p0::*;
use testcontainers::testcontainers::clients::Cli;

mod tests_async;
pub mod utils;

pub type C3p0Impl = SqlxSqliteC3p0Pool;

pub async fn new_connection(_docker: &Cli) -> (SqlxSqliteC3p0Pool, ()) {
    let options = SqliteConnectOptions::new();

    let pool: c3p0::sqlx::sqlx::Pool<Sqlite> = c3p0::sqlx::sqlx::pool::PoolOptions::new()
        .max_lifetime(None)
        .idle_timeout(None)
        .max_connections(1)
        .connect_with(options)
        .await
        .unwrap();

    let pool = SqlxSqliteC3p0Pool::new(pool);

    (pool, ())
}

pub mod db_specific {
    use super::*;

    pub fn db_type() -> utils::DbType {
        utils::DbType::Sqlite
    }
}
