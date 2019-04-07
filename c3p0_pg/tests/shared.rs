use lazy_static::lazy_static;
use maybe_single::MaybeSingle;
use r2d2::Pool;
use r2d2_postgres::{PostgresConnectionManager, TlsMode};
use serde_derive::{Deserialize, Serialize};
use testcontainers::*;

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct TestData {
    pub first_name: String,
    pub last_name: String,
}

lazy_static! {
    static ref DOCKER: clients::Cli = clients::Cli::default();
    pub static ref SINGLETON: MaybeSingle<(
        Pool<PostgresConnectionManager>,
        Container<'static, clients::Cli, images::postgres::Postgres>
    )> = MaybeSingle::new(|| init());
}

fn init() -> (
    Pool<PostgresConnectionManager>,
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
    let pool = r2d2::Pool::builder()
        .min_idle(Some(10))
        .build(manager)
        .unwrap();

    (pool, node)
}
