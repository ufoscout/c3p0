#![cfg(feature = "sqlx_mysql")]

use std::sync::Arc;

use c3p0::sqlx::sqlx::mysql::*;
use c3p0::sqlx::sqlx::Row;
use c3p0::sqlx::*;
use c3p0::*;
use maybe_single::tokio::{Data, MaybeSingleAsync};
use once_cell::sync::OnceCell;
use testcontainers::testcontainers::core::WaitFor;
use testcontainers::testcontainers::runners::AsyncRunner;
use testcontainers::testcontainers::ContainerAsync;
use testcontainers::testcontainers::GenericImage;

pub type C3p0Impl = SqlxMySqlC3p0Pool;
pub type Builder = SqlxMySqlC3p0JsonBuilder<u64>;
pub type UuidBuilder = SqlxMySqlC3p0JsonBuilder<uuid::Uuid>;

pub fn new_uuid_builder(table_name: &str) -> UuidBuilder {
    SqlxMySqlC3p0JsonBuilder::new(table_name).with_id_generator(Arc::new(MySqlUuidIdGenerator {}))
}

//mod tests;
mod tests_json;
mod utils;

pub type MaybeType = (C3p0Impl, ContainerAsync<GenericImage>);

async fn init() -> MaybeType {
    let mysql_version = "5.7.25";
    let mysql_image = GenericImage::new("mysql", mysql_version)
        .with_wait_for(WaitFor::message_on_stderr(
            format!("Version: '{}'  socket: '/var/run/mysqld/mysqld.sock'  port: 3306  MySQL Community Server (GPL)", mysql_version),
        ))
        .with_env_var("MYSQL_DATABASE", "mysql")
        .with_env_var("MYSQL_USER", "mysql")
        .with_env_var("MYSQL_PASSWORD", "mysql")
        .with_env_var("MYSQL_ROOT_PASSWORD", "mysql");

    let node = mysql_image.start().await;

    let options = MySqlConnectOptions::new()
        .username("mysql")
        .password("mysql")
        .database("mysql")
        .host("127.0.0.1")
        .port(node.get_host_port_ipv4(3306).await)
        .ssl_mode(MySqlSslMode::Disabled);

    let pool = MySqlPool::connect_with(options).await.unwrap();

    let pool = SqlxMySqlC3p0Pool::new(pool);

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
