#![cfg(feature = "postgres")]

use c3p0::postgres::deadpool::Runtime;
use c3p0::postgres::tokio_postgres::NoTls;
pub use c3p0::postgres::*;
pub use c3p0::*;
use std::time::Duration;
use testcontainers::{
    postgres::Postgres,
    testcontainers::{ContainerAsync, runners::AsyncRunner},
};

mod tests_async;
pub mod utils;

pub type C3p0Impl = PgC3p0Pool;
pub type C3p0JsonBuilder = PgC3p0JsonBuilder<u64, i64>;

pub async fn new_connection() -> (PgC3p0Pool, ContainerAsync<Postgres>) {
    let node = Postgres::default().start().await.unwrap();

    let mut config = deadpool::postgres::Config {
        user: Some("postgres".to_owned()),
        password: Some("postgres".to_owned()),
        dbname: Some("postgres".to_owned()),
        host: Some("127.0.0.1".to_string()),
        port: Some(node.get_host_port_ipv4(5432).await.unwrap()),
        ..Default::default()
    };

    let mut pool_config = deadpool::managed::PoolConfig::default();
    pool_config.timeouts.create = Some(Duration::from_secs(5));
    pool_config.timeouts.recycle = Some(Duration::from_secs(5));
    pool_config.timeouts.wait = Some(Duration::from_secs(5));
    config.pool = Some(pool_config);

    let pool = PgC3p0Pool::new(config.create_pool(Some(Runtime::Tokio1), NoTls).unwrap());

    (pool, node)
}

pub mod db_specific {
    use super::*;

    pub fn db_type() -> utils::DbType {
        utils::DbType::Pg
    }
}
