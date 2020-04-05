#![cfg(feature = "pg")]

use c3p0::pg::deadpool;
pub use c3p0::pg::tokio_postgres::{row::Row, NoTls};
use c3p0::pg::*;
use c3p0::*;
use maybe_single::{Data, MaybeSingleAsync};
use testcontainers::*;
use once_cell::sync::OnceCell;

use futures::FutureExt;
use std::time::Duration;

pub type C3p0Impl = PgC3p0PoolAsync;

mod tests_json;
mod utils;

pub type MaybeType = (
    C3p0Impl,
    Container<'static, clients::Cli, images::postgres::Postgres>,
);

async fn init() -> MaybeType {
    static DOCKER: OnceCell<clients::Cli> = OnceCell::new();
    let node = DOCKER
        .get_or_init(|| clients::Cli::default())
        .run(images::postgres::Postgres::default());

    let mut config = deadpool::postgres::Config::default();
    config.user = Some("postgres".to_owned());
    config.password = Some("postgres".to_owned());
    config.dbname = Some("postgres".to_owned());
    config.host = Some(format!("127.0.0.1"));
    config.port = Some(node.get_host_port(5432).unwrap());
    let mut pool_config = deadpool::managed::PoolConfig::default();
    pool_config.timeouts.create = Some(Duration::from_secs(5));
    pool_config.timeouts.recycle = Some(Duration::from_secs(5));
    pool_config.timeouts.wait = Some(Duration::from_secs(5));
    config.pool = Some(pool_config);

    let pool = PgC3p0PoolAsync::new(config.create_pool(NoTls).unwrap());

    (pool, node)
}

pub async fn data(serial: bool) -> Data<'static, MaybeType> {
    static DATA: OnceCell<MaybeSingleAsync<MaybeType>> = OnceCell::new();
    DATA.get_or_init(|| MaybeSingleAsync::new(|| init().boxed()))
        .data(serial)
        .await
}

pub mod db_specific {

    use super::*;

    pub fn db_type() -> utils::DbType {
        utils::DbType::Pg
    }
}
