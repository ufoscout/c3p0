#![cfg(feature = "tidb")]

/*
pub use c3p0::blocking::*;
use c3p0::mysql::blocking::mysql::{Opts, OptsBuilder};
use c3p0::mysql::blocking::r2d2::{MysqlConnectionManager, Pool};
pub use c3p0::mysql::blocking::*;
use testcontainers::*;

mod tests_blocking;
pub mod utils;

pub fn new_connection(
    docker: &clients::Cli,
) -> (
    MysqlC3p0Pool,
    Container<clients::Cli, images::generic::GenericImage>,
) {
    let mysql_version = "v3.0.3";
    let mysql_image = images::generic::GenericImage::new(format!("pingcap/tidb:{}", mysql_version))
        .with_wait_for(images::generic::WaitFor::message_on_stdout(
            r#"["server is running MySQL protocol"] [addr=0.0.0.0:4000]"#,
        ));
    let node = docker.run(mysql_image);

    let db_url = format!(
        "mysql://root@127.0.0.1:{}/mysql",
        node.get_host_port(4000).unwrap()
    );

    let opts = Opts::from_url(&db_url).unwrap();
    let builder = OptsBuilder::from_opts(opts);

    let manager = MysqlConnectionManager::new(builder);

    let pool = Pool::builder().min_idle(Some(10)).build(manager).unwrap();

    let pool = MysqlC3p0Pool::new(pool);

    (pool, node)
}

pub mod db_specific {
    use super::*;

    pub fn db_type() -> utils::DbType {
        utils::DbType::TiDB
    }
}


 */
