use c3p0_pg::Model;
use postgres::{Connection, TlsMode};
use serde_derive::{Deserialize, Serialize};
use testcontainers::*;

pub type TestModel = Model<TestData>;

#[derive(Clone, Serialize, Deserialize)]
pub struct TestData {
    pub first_name: String,
    pub last_name: String,
}


pub fn new_connection() -> Connection {
    let docker = clients::Cli::default();
    let node = docker.run(images::postgres::Postgres::default());

    Connection::connect(
        format!(
            "postgres://postgres:postgres@127.0.0.1:{}/postgres",
            node.get_host_port(5432).unwrap()
        ),
        TlsMode::None,
    )
    .unwrap()
}
