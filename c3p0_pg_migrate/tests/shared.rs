use postgres::{Connection, TlsMode};
use testcontainers::*;


pub fn new_connection() -> Connection {
    let docker = clients::Cli::default();
    let node = docker.run(images::postgres::Postgres::default());

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
