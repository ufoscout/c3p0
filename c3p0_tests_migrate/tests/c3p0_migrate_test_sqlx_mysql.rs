#![cfg(feature = "sqlx_mysql")]

use c3p0::sqlx::*;
use c3p0::sqlx::sqlx::mysql::*;
pub use c3p0::*;
use testcontainers::*;

mod tests_async;
pub mod utils;

pub type C3p0Impl = SqlxMySqlC3p0Pool;

pub async fn new_connection(
    docker: &clients::Cli,
) -> (
    SqlxMySqlC3p0Pool,
    Container<'_, clients::Cli, images::generic::GenericImage>,
) {
    let mysql_version = "5.7.25";
    let mysql_image = images::generic::GenericImage::new(format!("mysql:{}", mysql_version))
        .with_wait_for(images::generic::WaitFor::message_on_stderr(
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
        .port(node.get_host_port(3306).unwrap())
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
