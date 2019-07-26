#![cfg(feature = "pg")]

use c3p0_common::C3p0Pool;
use c3p0_pool_pg::r2d2::{Pool, PostgresConnectionManager, TlsMode};
use c3p0_pool_pg::PgPoolManager;
use testcontainers::*;

mod tests;

pub fn new_connection(
    docker: &clients::Cli,
) -> (
    C3p0Pool<PgPoolManager>,
    Container<clients::Cli, images::postgres::Postgres>,
) {
    let node = docker.run(images::postgres::Postgres::default());

    let manager = PostgresConnectionManager::new(
        format!(
            "postgres://postgres:postgres@127.0.0.1:{}/postgres",
            node.get_host_port(5432).unwrap()
        ),
        TlsMode::None,
    )
    .unwrap();
    let pool = Pool::builder().min_idle(Some(10)).build(manager).unwrap();

    let pool = C3p0Pool::new(PgPoolManager::new(pool));

    (pool, node)
}
