#![cfg(feature = "pg_async")]

use c3p0_pg_async::pg_async::driver::tls::NoTls;
use c3p0_pg_async::pg_async::bb8::{Pool, PostgresConnectionManager};
use c3p0_pg_async::pg_async::*;
use c3p0_pg_async::*;
use testcontainers::*;

mod tests_async;

pub async fn new_connection(
    docker: &clients::Cli,
) -> (
    PgC3p0PoolAsync,
    Container<'_, clients::Cli, images::postgres::Postgres>,
) {
    let node = docker.run(images::postgres::Postgres::default());
    let manager = PostgresConnectionManager::new(
        format!(
            "postgres://postgres:postgres@127.0.0.1:{}/postgres",
            node.get_host_port(5432).unwrap()
        )
        .parse()
        .unwrap(),
        NoTls,
    );

    let pool = Pool::builder()
        .min_idle(Some(10))
        .build(manager)
        .await
        .unwrap();
    let pool = PgC3p0PoolAsync::new(pool);

    (pool, node)
}
