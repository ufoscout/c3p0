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

pub fn postgres_image() -> images::generic::GenericImage {
    images::generic::GenericImage::new("postgres:11-alpine").with_wait_for(
        images::generic::WaitFor::message_on_stderr(
            "database system is ready to accept connections",
        ),
    )
}

pub fn new_connection() -> Connection {
    let docker = clients::Cli::default();
    let node = docker.run(postgres_image());

    let conn = Connection::connect(
        format!(
            "postgres://postgres:postgres@127.0.0.1:{}/postgres",
            node.get_host_port(5432).unwrap()
        ),
        TlsMode::None,
    )
    .unwrap();

    conn.batch_execute(
        "create table TEST_TABLE (
                            ID bigserial primary key,
                            VERSION int not null,
                            DATA JSONB
                        );

                create table TEST_TABLE_2 (
                            ID bigserial primary key,
                            VERSION int not null,
                            DATA JSONB
                        );

                        ",
    )
    .unwrap();

    conn
}
