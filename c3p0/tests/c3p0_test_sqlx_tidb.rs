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
    let tidb_version = "v3.0.3";
    let tidb_image = GenericImage::new("pingcap/tidb", tidb_version).with_wait_for(
        WaitFor::message_on_stdout(r#"["server is running MySQL protocol"] [addr=0.0.0.0:4000]"#),
    );

    let node = tidb_image.start().await;

    let options = MySqlConnectOptions::new()
        .username("root")
        //.password("mysql")
        .database("mysql")
        .host("127.0.0.1")
        .port(node.get_host_port_ipv4(4000).await)
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
        utils::DbType::TiDB
    }

    pub fn row_to_string(row: &MySqlRow) -> Result<String, Box<dyn std::error::Error>> {
        let value: String = row.get(0);
        Ok(value)
    }

    pub fn build_insert_query(table_name: &str) -> String {
        format!(r"INSERT INTO {} (name) VALUES (?)", table_name)
    }
}
