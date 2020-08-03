#![cfg(feature = "tidb")]
pub use c3p0::tidb::mysql_async::Row;
use c3p0::*;

mod utils;

/*
use c3p0::blocking::*;
pub use c3p0::mysql::blocking::mysql::{Opts, OptsBuilder, Row};
use c3p0::mysql::blocking::r2d2::{MysqlConnectionManager, Pool};
use c3p0::mysql::blocking::*;
use maybe_single::{Data, MaybeSingle};
use once_cell::sync::OnceCell;
use testcontainers::*;

pub type C3p0Impl = MysqlC3p0Pool;

mod tests_blocking;
mod tests_blocking_json;

pub type MaybeType = (
    C3p0Impl,
    Container<'static, clients::Cli, images::generic::GenericImage>,
);

fn init() -> MaybeType {
    static DOCKER: OnceCell<clients::Cli> = OnceCell::new();

    let tidb_version = "v3.0.3";
    let tidb_image = images::generic::GenericImage::new(format!("pingcap/tidb:{}", tidb_version))
        .with_wait_for(images::generic::WaitFor::message_on_stdout(
            r#"["server is running MySQL protocol"] [addr=0.0.0.0:4000]"#,
        ));
    let node = DOCKER
        .get_or_init(|| clients::Cli::default())
        .run(tidb_image);

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

pub fn data(serial: bool) -> Data<'static, MaybeType> {
    static DATA: OnceCell<MaybeSingle<MaybeType>> = OnceCell::new();
    DATA.get_or_init(|| MaybeSingle::new(|| init()))
        .data(serial)
}

*/
pub mod db_specific {

    use super::*;

    pub fn db_type() -> utils::DbType {
        utils::DbType::TiDB
    }

    pub fn row_to_string(row: &Row) -> Result<String, Box<dyn std::error::Error>> {
        Ok(row.get(0).ok_or_else(|| C3p0Error::ResultNotFoundError)?)
    }

    pub fn build_insert_query(table_name: &str) -> String {
        format!(r"INSERT INTO {} (name) VALUES (?)", table_name)
    }
}
