#![cfg(feature = "postgres")]

use c3p0::postgres::deadpool;
pub use c3p0::postgres::tokio_postgres::{row::Row, NoTls};
use c3p0::postgres::*;
use c3p0::*;
use c3p0_postgres::deadpool::Runtime;
use maybe_single::tokio::{Data, MaybeSingleAsync};
use once_cell::sync::OnceCell;
use testcontainers::{
    postgres::Postgres,
    testcontainers::{clients::Cli, Container},
};

use std::time::Duration;

pub type C3p0Impl = PgC3p0Pool;
pub type Builder = PgC3p0JsonBuilder<u64, i64>;
pub type UuidBuilder = PgC3p0JsonBuilder<uuid::Uuid, uuid::Uuid>;

pub fn new_uuid_builder(table_name: &str) -> UuidBuilder {
    PgC3p0JsonBuilder::new(table_name).with_id_generator(UuidIdGenerator {})
}

mod tests;
mod tests_json;
mod utils;

pub type MaybeType = (C3p0Impl, Container<'static, Postgres>);

async fn init() -> MaybeType {
    static DOCKER: OnceCell<Cli> = OnceCell::new();
    let node = DOCKER.get_or_init(Cli::default).run(Postgres::default());

    let mut config = deadpool::postgres::Config {
        user: Some("postgres".to_owned()),
        password: Some("postgres".to_owned()),
        dbname: Some("postgres".to_owned()),
        host: Some("127.0.0.1".to_string()),
        port: Some(node.get_host_port_ipv4(5432)),
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

pub async fn data(serial: bool) -> Data<'static, MaybeType> {
    static DATA: OnceCell<MaybeSingleAsync<MaybeType>> = OnceCell::new();
    DATA.get_or_init(|| MaybeSingleAsync::new(|| Box::pin(init())))
        .data(serial)
        .await
}

pub mod db_specific {

    use super::*;

    pub fn db_type() -> utils::DbType {
        utils::DbType::Pg
    }

    pub fn row_to_string(row: &Row) -> Result<String, Box<dyn std::error::Error>> {
        let value: String = row.get(0);
        Ok(value)
    }

    pub fn build_insert_query(table_name: &str) -> String {
        format!(r"INSERT INTO {} (name) VALUES ($1)", table_name)
    }
}
