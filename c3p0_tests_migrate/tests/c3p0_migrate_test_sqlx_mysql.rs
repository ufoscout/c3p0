#![cfg(feature = "sqlx_mysql")]

use c3p0::sqlx::sqlx::mysql::*;
use c3p0::sqlx::*;
pub use c3p0::*;
use testcontainers::testcontainers::{clients::Cli, core::WaitFor, Container, GenericImage};

mod tests_async;
pub mod utils;

pub type C3p0Impl = SqlxMySqlC3p0Pool;

pub async fn new_connection(docker: &Cli) -> (SqlxMySqlC3p0Pool, Container<'_, GenericImage>) {
    let mysql_version = "5.7.25";
    let mysql_image = GenericImage::new("mysql", mysql_version)
        .with_wait_for(WaitFor::message_on_stderr(
            format!("Version: '{}'  socket: '/var/run/mysqld/mysqld.sock'  port: 3306  MySQL Community Server (GPL)", mysql_version),
        ))
        .with_env_var("MYSQL_DATABASE", "mysql")
        .with_env_var("MYSQL_USER", "mysql")
        .with_env_var("MYSQL_PASSWORD", "mysql")
        .with_env_var("MYSQL_ROOT_PASSWORD", "mysql");

    let node = docker.run(mysql_image);

    let options = MySqlConnectOptions::new()
        .username("mysql")
        .password("mysql")
        .database("mysql")
        .host("127.0.0.1")
        .port(node.get_host_port_ipv4(3306))
        .ssl_mode(MySqlSslMode::Disabled);

    let pool = MySqlPool::connect_with(options).await.unwrap();

    let pool = SqlxMySqlC3p0Pool::new(pool);

    (pool, node)
}

pub mod db_specific {
    use super::*;

    pub fn db_type() -> utils::DbType {
        utils::DbType::MySql
    }
}
