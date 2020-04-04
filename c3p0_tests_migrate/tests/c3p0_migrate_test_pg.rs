#![cfg(feature = "pg")]

use c3p0::pg::tokio_postgres::NoTls;
use c3p0::pg::deadpool;
pub use c3p0::pg::*;
pub use c3p0::*;
use testcontainers::*;
use std::time::Duration;

mod tests;
pub mod utils;

pub async fn new_connection(
    docker: &clients::Cli,
) -> (
    PgC3p0PoolAsync,
    Container<'_, clients::Cli, images::postgres::Postgres>,
) {
    let node = docker.run(images::postgres::Postgres::default());

    let mut config = deadpool::postgres::Config::default();
    config.user = Some("postgres".to_owned());
    config.password = Some("postgres".to_owned());
    config.dbname = Some("postgres".to_owned());
    config.host = Some(format!("127.0.0.1"));
    config.port = Some(node.get_host_port(5432).unwrap());
    let mut pool_config = deadpool::managed::PoolConfig::default();
    pool_config.timeouts.create = Some(Duration::from_secs(5));
    pool_config.timeouts.recycle = Some(Duration::from_secs(5));
    pool_config.timeouts.wait = Some(Duration::from_secs(5));
    config.pool = Some(pool_config);

    let pool = PgC3p0PoolAsync::new(config.create_pool(NoTls).unwrap());

    (pool, node)
}
