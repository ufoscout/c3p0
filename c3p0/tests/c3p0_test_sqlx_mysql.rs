#![cfg(feature = "sqlx_mysql")]

use std::sync::Arc;
use std::sync::OnceLock;

use c3p0::sqlx::sqlx::Row;
use c3p0::sqlx::sqlx::mysql::*;
use c3p0::sqlx::*;
use c3p0::*;
use maybe_single::tokio::{Data, MaybeSingleAsync};
use testcontainers::mysql::Mysql;
use testcontainers::testcontainers::ContainerAsync;
use testcontainers::testcontainers::runners::AsyncRunner;

pub type C3p0Impl = SqlxMySqlC3p0Pool;
pub type Builder = SqlxMySqlC3p0JsonBuilder<u64>;
pub type UuidBuilder = SqlxMySqlC3p0JsonBuilder<uuid::Uuid>;

pub fn new_uuid_builder(table_name: &str) -> UuidBuilder {
    SqlxMySqlC3p0JsonBuilder::new(table_name).with_id_generator(Arc::new(MySqlUuidIdGenerator {}))
}

//mod tests;
mod tests_json;
mod utils;

pub type MaybeType = (C3p0Impl, ContainerAsync<Mysql>);

async fn init() -> MaybeType {
    let node = Mysql::default().start().await.unwrap();

    let options = MySqlConnectOptions::new()
        // .username("mysql")
        // .password("mysql")
        .database("test")
        .host("127.0.0.1")
        .port(node.get_host_port_ipv4(3306).await.unwrap())
        .ssl_mode(MySqlSslMode::Disabled);

    let pool = MySqlPool::connect_with(options).await.unwrap();

    let pool = SqlxMySqlC3p0Pool::new(pool);

    (pool, node)
}

pub async fn data(serial: bool) -> Data<'static, MaybeType> {
    static DATA: OnceLock<MaybeSingleAsync<MaybeType>> = OnceLock::new();
    DATA.get_or_init(|| MaybeSingleAsync::new(|| Box::pin(init())))
        .data(serial)
        .await
}

pub mod db_specific {

    use super::*;

    pub fn db_type() -> utils::DbType {
        utils::DbType::MySql
    }

    pub fn row_to_string(row: &MySqlRow) -> Result<String, Box<dyn std::error::Error>> {
        let value: String = row.get(0);
        Ok(value)
    }

    pub fn build_insert_query(table_name: &str) -> String {
        format!(r"INSERT INTO {} (name) VALUES (?)", table_name)
    }
}
