#![cfg(feature = "pg")]

use c3p0_json::pg::r2d2::{PostgresConnectionManager, TlsMode, Pool};
use testcontainers::*;

pub use c3p0_json::pg::PgPoolManager as C3p0Impl;
pub use c3p0_json::pg::C3p0PgBuilder as C3p0BuilderImpl;

mod tests;

pub fn new_connection(
    docker: &clients::Cli,
) -> (C3p0Impl, Container<clients::Cli, images::postgres::Postgres>) {
    let node = docker.run(images::postgres::Postgres::default());

    let manager = PostgresConnectionManager::new(
        format!(
            "postgres://postgres:postgres@127.0.0.1:{}/postgres",
            node.get_host_port(5432).unwrap()
        ),
        TlsMode::None,
    )
    .unwrap();
    let pool = Pool::builder()
        .min_idle(Some(10))
        .build(manager)
        .unwrap();

    let pool = C3p0BuilderImpl::build(pool);

    (pool, node)
}
