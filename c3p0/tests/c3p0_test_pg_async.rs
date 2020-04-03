#![cfg(feature = "pg_async")]

use c3p0::pg_async::deadpool;
use c3p0::pg_async::*;
use c3p0::*;
use lazy_static::lazy_static;
use maybe_single::nio::{Data, MaybeSingleAsync};
use testcontainers::*;

pub use c3p0::pg_async::driver::row::Row;
pub use c3p0::pg_async::driver::tls::NoTls;
use futures::FutureExt;
use std::time::Duration;

pub type C3p0Impl = PgC3p0PoolAsync;

//mod tests;
mod tests_json_async;

pub mod utils;

lazy_static! {
    static ref DOCKER: clients::Cli = clients::Cli::default();
    pub static ref SINGLETON: MaybeSingleAsync<MaybeType> =
        MaybeSingleAsync::new(|| init().boxed());
}

pub type MaybeType = (
    C3p0Impl,
    Container<'static, clients::Cli, images::postgres::Postgres>,
);

async fn init() -> MaybeType {
    let node = DOCKER.run(images::postgres::Postgres::default());

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
    SINGLETON.data(serial).await
}
