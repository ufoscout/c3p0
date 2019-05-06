use c3p0::prelude::*;
use r2d2_postgres::{PostgresConnectionManager, TlsMode};
use testcontainers::*;

pub fn new_connection(
    docker: &clients::Cli,
) -> (C3p0, Container<clients::Cli, images::postgres::Postgres>) {
    let node = docker.run(images::postgres::Postgres::default());

    let manager = PostgresConnectionManager::new(
        format!(
            "postgres://postgres:postgres@127.0.0.1:{}/postgres",
            node.get_host_port(5432).unwrap()
        ),
        TlsMode::None,
    )
    .unwrap();
    let pool = r2d2::Pool::builder()
        .min_idle(Some(10))
        .build(manager)
        .unwrap();

    let pool = C3p0Builder::build(pool);

    (pool, node)
}
