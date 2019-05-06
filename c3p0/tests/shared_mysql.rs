#![cfg(feature = "mysql")]

use c3p0::prelude::*;
use lazy_static::lazy_static;
use maybe_single::MaybeSingle;
use mysql_client::{Opts, OptsBuilder};
use r2d2_mysql::MysqlConnectionManager;
use serde_derive::{Deserialize, Serialize};
use testcontainers::*;

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct TestData {
    pub first_name: String,
    pub last_name: String,
}

lazy_static! {
    static ref DOCKER: clients::Cli = clients::Cli::default();
    pub static ref SINGLETON: MaybeSingle<(
        C3p0,
        Container<'static, clients::Cli, images::generic::GenericImage>
    )> = MaybeSingle::new(|| init());
}

fn init() -> (
    C3p0,
    Container<'static, clients::Cli, images::generic::GenericImage>,
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
    let node = DOCKER.run(mysql_image);

    let db_url = format!(
        "mysql://mysql:mysql@127.0.0.1:{}/mysql",
        node.get_host_port(3306).unwrap()
    );

    let opts = Opts::from_url(&db_url).unwrap();
    let builder = OptsBuilder::from_opts(opts);

    let manager = MysqlConnectionManager::new(builder);

    let pool = r2d2::Pool::builder()
        .min_idle(Some(10))
        .build(manager)
        .unwrap();

    let pool = C3p0Builder::build(pool);

    (pool, node)
}
