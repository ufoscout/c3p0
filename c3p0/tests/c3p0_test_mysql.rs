#![cfg(feature = "mysql")]
pub use c3p0::mysql::mysql_async::Row;
use c3p0::*;
use c3p0::mysql::*;
use maybe_single::{Data, MaybeSingleAsync};
use once_cell::sync::OnceCell;
use testcontainers::*;
use c3p0_mysql::mysql_async::{Opts, Pool};

pub type C3p0Impl = MysqlC3p0Pool;

mod tests;
mod tests_json;
mod utils;

pub type MaybeType = (
    C3p0Impl,
    Container<'static, clients::Cli, images::generic::GenericImage>,
);

async fn init() -> MaybeType {
    static DOCKER: OnceCell<clients::Cli> = OnceCell::new();

    let mysql_version = "5.7.25";
    let mysql_image = images::generic::GenericImage::new(format!("mysql:{}", mysql_version))
        .with_wait_for(images::generic::WaitFor::message_on_stderr(
            format!("Version: '{}'  socket: '/var/run/mysqld/mysqld.sock'  port: 3306  MySQL Community Server (GPL)", mysql_version),
        ))
        .with_env_var("MYSQL_DATABASE", "mysql")
        .with_env_var("MYSQL_USER", "mysql")
        .with_env_var("MYSQL_PASSWORD", "mysql")
        .with_env_var("MYSQL_ROOT_PASSWORD", "mysql");

    let node = DOCKER
        .get_or_init(|| clients::Cli::default())
        .run(mysql_image);

    let db_url = format!(
        "mysql://mysql:mysql@127.0.0.1:{}/mysql",
        node.get_host_port(3306).unwrap()
    );

    let opts = Opts::from_url(&db_url).unwrap();
//    let builder = OptsBuilder::from_opts(opts);

    let pool = Pool::new(opts);

    let pool = MysqlC3p0Pool::new(pool);

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

    pub fn row_to_string(row: &Row) -> Result<String, Box<dyn std::error::Error>> {
        Ok(row.get(0).ok_or_else(|| C3p0Error::ResultNotFoundError)?)
    }

    pub fn build_insert_query(table_name: &str) -> String {
        format!(r"INSERT INTO {} (name) VALUES (?)", table_name)
    }
}
