#![cfg(feature = "mysql_blocking")]

pub use c3p0::mysql::blocking::mysql::{Opts, OptsBuilder, Row};
use c3p0::mysql::blocking::r2d2::{MysqlConnectionManager, Pool};
use c3p0::mysql::blocking::*;
use c3p0::blocking::*;
use lazy_static::lazy_static;
use maybe_single::{Data, MaybeSingle};
use testcontainers::*;

pub type C3p0Impl = MysqlC3p0Pool;

mod tests_blocking;
mod tests_blocking_json;
mod utils;

lazy_static! {
    static ref DOCKER: clients::Cli = clients::Cli::default();
    pub static ref SINGLETON: MaybeSingle<MaybeType> = MaybeSingle::new(|| init());
}

pub type MaybeType = (
    C3p0Impl,
    Container<'static, clients::Cli, images::generic::GenericImage>,
);

fn init() -> MaybeType {
    let mysql_version = "5.7.25";
    let mysql_image = images::generic::GenericImage::new(format!("mysql:{}", mysql_version))
        .with_wait_for(images::generic::WaitFor::message_on_stderr(
            format!("Version: '{}'  socket: '/var/run/mysqld/mysqld.sock'  port: 3306  MySQL Community Server (GPL)", mysql_version),
        ))
        .with_env_var("MYSQL_DATABASE", "mysql")
        .with_env_var("MYSQL_USER", "mysql")
        .with_env_var("MYSQL_PASSWORD", "mysql")
        .with_env_var("MYSQL_ROOT_PASSWORD", "mysql");
    let node = DOCKER.run(mysql_image);

    let db_url = format!(
        "mysql://mysql:mysql@127.0.0.1:{}/mysql",
        node.get_host_port(3306).unwrap()
    );

    let opts = Opts::from_url(&db_url).unwrap();
    let builder = OptsBuilder::from_opts(opts);

    let manager = MysqlConnectionManager::new(builder);

    let pool = Pool::builder().min_idle(Some(10)).build(manager).unwrap();

    let pool = MysqlC3p0Pool::new(pool);

    (pool, node)
}

pub fn data(serial: bool) -> Data<'static, MaybeType> {
    SINGLETON.data(serial)
}

pub mod db_specific {

    use super::*;

    pub fn row_to_string(row: &Row) -> Result<String, Box<dyn std::error::Error>> {
        Ok(row.get(0).ok_or_else(|| C3p0Error::ResultNotFoundError)?)
    }

    pub fn build_insert_query(table_name: &str) -> String {
        format!(r"INSERT INTO {} (name) VALUES (?)", table_name)
    }

}