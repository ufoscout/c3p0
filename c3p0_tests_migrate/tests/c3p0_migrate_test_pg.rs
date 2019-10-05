#![cfg(feature = "pg")]

use c3p0_pg::pg::r2d2::{Pool, PostgresConnectionManager, TlsMode};
use c3p0_pg::pg::*;
use c3p0_pg::*;
use testcontainers::*;

mod tests;

pub fn new_connection(
    docker: &clients::Cli,
) -> (
    PgC3p0Pool,
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

    let pool = PgC3p0Pool::new(pool);

    (pool, node)
}
