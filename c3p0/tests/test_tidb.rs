#![cfg(feature = "mysql")]

use c3p0::mysql::driver::{Opts, OptsBuilder};
use c3p0::mysql::r2d2::{MysqlConnectionManager, Pool};
use c3p0::mysql::*;
use c3p0::*;
use lazy_static::lazy_static;
use maybe_single::MaybeSingle;
use serde_derive::{Deserialize, Serialize};
use testcontainers::*;

pub use c3p0::mysql::driver::Row;

pub type C3p0Impl = C3p0PoolMysql;

mod tests;

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct TestData {
    pub first_name: String,
    pub last_name: String,
}

lazy_static! {
    static ref DOCKER: clients::Cli = clients::Cli::default();
    pub static ref SINGLETON: MaybeSingle<(
        C3p0Impl,
        Container<'static, clients::Cli, images::generic::GenericImage>
    )> = MaybeSingle::new(|| init());
}

fn init() -> (
    C3p0Impl,
    Container<'static, clients::Cli, images::generic::GenericImage>,
) {
    let mysql_version = "v3.0.3";
    let mysql_image = images::generic::GenericImage::new(format!("pingcap/tidb:{}", mysql_version))
        .with_wait_for(images::generic::WaitFor::message_on_stdout(
            r#"["server is running MySQL protocol"] [addr=0.0.0.0:4000]"#,
        ));
    let node = DOCKER.run(mysql_image);

    let db_url = format!(
        "mysql://root@127.0.0.1:{}/mysql",
        node.get_host_port(4000).unwrap()
    );

    let opts = Opts::from_url(&db_url).unwrap();
    let builder = OptsBuilder::from_opts(opts);

    let manager = MysqlConnectionManager::new(builder);

    let pool = Pool::builder().min_idle(Some(10)).build(manager).unwrap();

    let pool = C3p0PoolMysql::new(pool);

    (pool, node)
}
