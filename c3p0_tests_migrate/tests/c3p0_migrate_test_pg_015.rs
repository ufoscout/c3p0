#![cfg(feature = "pg_015")]

use c3p0_pg_015::pg::r2d2::{Pool, PostgresConnectionManager, TlsMode};
use c3p0_pg_015::pg::*;
use c3p0_pg_015::*;
use testcontainers::*;

mod tests;

pub fn new_connection(
    docker: &clients::Cli,
) -> (
    PgC3p0Pool,
    Container<clients::Cli, images::generic::GenericImage>,
) {
    let node = docker.run(
        images::generic::GenericImage::new("postgres:11-alpine")
            .with_wait_for(images::generic::WaitFor::message_on_stderr(
                "database system is ready to accept connections",
            ))
            .with_env_var("POSTGRES_DB", "postgres")
            .with_env_var("POSTGRES_USER", "postgres")
            .with_env_var("POSTGRES_PASSWORD", "postgres"),
    );

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
