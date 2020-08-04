#![cfg(feature = "sqlx_postgres")]

use c3p0::sqlx::*;
use c3p0::sqlx::sqlx::postgres::*;
use c3p0::sqlx::sqlx::Row;
pub use c3p0::*;
use testcontainers::*;

mod tests_async;
pub mod utils;

pub async fn new_connection(
    docker: &clients::Cli,
) -> (
    SqlxPgC3p0Pool,
    Container<'_, clients::Cli, images::postgres::Postgres>,
) {
    let node = docker.run(images::postgres::Postgres::default());

    let options = PgConnectOptions::new()
        .username("postgres")
        .password("postgres")
        .database("postgres")
        .host("127.0.0.1")
        .port(node.get_host_port(5432).unwrap());

    let pool = PgPool::connect_with(options).await.unwrap();

    let pool = SqlxPgC3p0Pool::new(pool);

    (pool, node)
}

pub mod db_specific {
    use super::*;

    pub fn db_type() -> utils::DbType {
        utils::DbType::Pg
    }
}
