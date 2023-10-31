#![cfg(feature = "sqlx_mysql")]

use c3p0::sqlx::sqlx::mysql::*;
use c3p0::sqlx::*;
pub use c3p0::*;
use ::testcontainers::testcontainers::Container;
use testcontainers::testcontainers::GenericImage;
use testcontainers::testcontainers::clients::Cli;
use testcontainers::testcontainers::core::WaitFor;

mod tests_async;
pub mod utils;

pub type C3p0Impl = SqlxMySqlC3p0Pool;

pub async fn new_connection(
    docker: &Cli,
) -> (
    SqlxMySqlC3p0Pool,
    Container<'_, GenericImage>,
) {
    let tidb_version = "v3.0.3";
    let tidb_image = GenericImage::new("pingcap/tidb", tidb_version)
        .with_wait_for(WaitFor::message_on_stdout(
            r#"["server is running MySQL protocol"] [addr=0.0.0.0:4000]"#,
        ));

    let node = docker.run(tidb_image);

    let options = MySqlConnectOptions::new()
        .username("root")
        //.password("mysql")
        .database("mysql")
        .host("127.0.0.1")
        .port(node.get_host_port_ipv4(4000))
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
