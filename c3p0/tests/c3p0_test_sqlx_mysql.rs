#![cfg(feature = "sqlx_mysql")]

use c3p0::sqlx::mysql::*;
use c3p0::sqlx::sqlx::mysql::*;
use c3p0::sqlx::sqlx::Row;
use c3p0::*;
use maybe_single::{Data, MaybeSingleAsync};
use once_cell::sync::OnceCell;
use testcontainers::*;

use futures::FutureExt;

pub type C3p0Impl = SqlxC3p0Pool;

//mod tests;
mod tests_json;
mod utils;

pub type MaybeType = (
    C3p0Impl,
    Container<'static, clients::Cli, images::generic::GenericImage>,
);

async fn init() -> MaybeType {
    let mysql_version = "5.7.25";
    let mysql_image = images::generic::GenericImage::new(format!("mysql:{}", mysql_version))
        .with_wait_for(images::generic::WaitFor::message_on_stderr(
            format!("Version: '{}'  socket: '/var/run/mysqld/mysqld.sock'  port: 3306  MySQL Community Server (GPL)", mysql_version),
        ))
        .with_env_var("MYSQL_DATABASE", "mysql")
        .with_env_var("MYSQL_USER", "mysql")
        .with_env_var("MYSQL_PASSWORD", "mysql")
        .with_env_var("MYSQL_ROOT_PASSWORD", "mysql");

    static DOCKER: OnceCell<clients::Cli> = OnceCell::new();
    let node = DOCKER
        .get_or_init(|| clients::Cli::default())
        .run(mysql_image);

    let options = MySqlConnectOptions::new()
        .username("mysql")
        .password("mysql")
        .database("mysql")
        .host("127.0.0.1")
        .port(node.get_host_port(3306).unwrap())
        .ssl_mode(MySqlSslMode::Disabled);

    let pool = MySqlPool::connect_with(options).await.unwrap();

    let pool = SqlxC3p0Pool::new(pool);

    (pool, node)
}

pub async fn data(serial: bool) -> Data<'static, MaybeType> {
    static DATA: OnceCell<MaybeSingleAsync<MaybeType>> = OnceCell::new();
    DATA.get_or_init(|| MaybeSingleAsync::new(|| init().boxed()))
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
