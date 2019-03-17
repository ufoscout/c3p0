use lazy_static::lazy_static;
use maybe_single::MaybeSingle;
use postgres::Connection;
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
        postgres::Connection,
        Container<'static, clients::Cli, images::postgres::Postgres>
    )> = MaybeSingle::new(|| init());
}

fn init() -> (
    postgres::Connection,
    Container<'static, clients::Cli, images::postgres::Postgres>,
) {
    let node = DOCKER.run(images::postgres::Postgres::default());

    let conn = Connection::connect(
        format!(
            "postgres://postgres:postgres@127.0.0.1:{}/postgres",
            node.get_host_port(5432).unwrap()
        ),
        postgres::TlsMode::None,
    )
    .unwrap();

    (conn, node)
}
