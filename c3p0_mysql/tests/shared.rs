use lazy_static::lazy_static;
use maybe_single::MaybeSingle;
use mysql::{Opts, OptsBuilder};
use r2d2::Pool;
use r2d2_mysql::MysqlConnectionManager;
use serde_derive::{Deserialize, Serialize};
use testcontainers::*;

#[derive(Clone, Serialize, Deserialize)]
pub struct TestData {
    pub first_name: String,
    pub last_name: String,
}

lazy_static! {
    static ref DOCKER: clients::Cli = clients::Cli::default();
    pub static ref SINGLETON: MaybeSingle<(
        Pool<MysqlConnectionManager>,
        Container<'static, clients::Cli, images::generic::GenericImage>
    )> = MaybeSingle::new(|| init());
}

fn init() -> (
    Pool<MysqlConnectionManager>,
    Container<'static, clients::Cli, images::generic::GenericImage>,
) {
    let mysql_image = images::generic::GenericImage::new("mysql:5.7")
        .with_wait_for(images::generic::WaitFor::message_on_stderr(
            "Version: '5.7.25'  socket: '/var/run/mysqld/mysqld.sock'  port: 3306  MySQL Community Server (GPL)",
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

    (pool, node)
}
