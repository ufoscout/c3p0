#![cfg(feature = "mysql")]

use std::sync::OnceLock;

use c3p0::*;
use maybe_once::tokio::{Data, MaybeOnceAsync};
use ::sqlx::mysql::{MySqlConnectOptions, MySqlSslMode};
use ::sqlx::{MySqlPool, Row};
use testcontainers::mysql::Mysql;
use testcontainers::testcontainers::ContainerAsync;
use testcontainers::testcontainers::runners::AsyncRunner;

pub type C3p0Impl = MySqlC3p0Pool;

mod tests;
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

    let pool = MySqlC3p0Pool::new(pool);

    (pool, node)
}

pub async fn data(serial: bool) -> Data<'static, MaybeType> {
    static DATA: OnceLock<MaybeOnceAsync<MaybeType>> = OnceLock::new();
    DATA.get_or_init(|| MaybeOnceAsync::new(|| Box::pin(init())))
        .data(serial)
        .await
}

pub mod db_specific {

    use ::sqlx::mysql::MySqlRow;

    use super::*;

    pub fn db_type() -> utils::DbType {
        utils::DbType::MySql
    }

    pub fn row_to_string(row: &MySqlRow) -> Result<String, Box<dyn std::error::Error>> {
        let value: String = row.get(0);
        Ok(value)
    }

    pub fn build_insert_query(table_name: &str) -> String {
        format!(r"INSERT INTO {table_name} (name) VALUES (?)")
    }
}
