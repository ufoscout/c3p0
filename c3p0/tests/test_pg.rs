#![cfg(feature = "pg")]

use c3p0::pg::r2d2::{Pool, PostgresConnectionManager, TlsMode};
use c3p0::*;
use lazy_static::lazy_static;
use maybe_single::MaybeSingle;
use serde_derive::{Deserialize, Serialize};
use testcontainers::*;

pub use c3p0::pg::postgres::rows::Row;

pub type C3p0Impl = C3p0Pool<PgPoolManager>;

mod tests;

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct TestData {
    pub first_name: String,
    pub last_name: String,
}

lazy_static! {
    static ref DOCKER: clients::Cli = clients::Cli::default();
    pub static ref SINGLETON: MaybeSingle<(
        C3p0Impl,
        Container<'static, clients::Cli, images::postgres::Postgres>
    )> = MaybeSingle::new(|| init());
}

fn init() -> (
    C3p0Impl,
    Container<'static, clients::Cli, images::postgres::Postgres>,
) {
    let node = DOCKER.run(images::postgres::Postgres::default());

    let manager = PostgresConnectionManager::new(
        format!(
            "postgres://postgres:postgres@127.0.0.1:{}/postgres",
            node.get_host_port(5432).unwrap()
        ),
        TlsMode::None,
    )
    .unwrap();
    let pool = Pool::builder().min_idle(Some(10)).build(manager).unwrap();

    let pool = C3p0Pool::new(PgPoolManager::new(pool));

    (pool, node)
}
