#![cfg(feature = "sqlx_postgres")]

use c3p0::sqlx::sqlx::postgres::*;
use c3p0::sqlx::*;
pub use c3p0::*;
use testcontainers::postgres::Postgres;
use testcontainers::testcontainers::clients::Cli;
use testcontainers::testcontainers::Container;

mod tests_async;
pub mod utils;

pub type C3p0Impl = SqlxPgC3p0Pool;

pub async fn new_connection(docker: &Cli) -> (SqlxPgC3p0Pool, Container<'_, Postgres>) {
    let node = docker.run(Postgres::default());

    let options = PgConnectOptions::new()
        .username("postgres")
        .password("postgres")
        .database("postgres")
        .host("127.0.0.1")
        .port(node.get_host_port_ipv4(5432));

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
