use lazy_static::lazy_static;
use r2d2::{Pool, PooledConnection};
use r2d2_postgres::PostgresConnectionManager;
use serde_derive::{Deserialize, Serialize};
use std::sync::Mutex;
use testcontainers::*;

#[derive(Clone, Serialize, Deserialize)]
pub struct TestData {
    pub first_name: String,
    pub last_name: String,
}

lazy_static! {
    static ref DOCKER: clients::Cli = clients::Cli::default();
    static ref CONTAINER: Mutex<Container<'static, clients::Cli, images::postgres::Postgres>> =
        Mutex::new(create_postgres_container());
    static ref POOL: Mutex<Pool<PostgresConnectionManager>> = Mutex::new(create_pool());
    pub static ref LOCK: Mutex<()> = Mutex::new(());
}

pub fn new_connection() -> PooledConnection<PostgresConnectionManager> {
    POOL.lock().unwrap().get().unwrap()
}

fn create_postgres_container<'a>() -> Container<'a, clients::Cli, images::postgres::Postgres> {
    DOCKER.run(images::postgres::Postgres::default())
}

fn create_pool() -> Pool<PostgresConnectionManager> {
    let node = CONTAINER.lock().unwrap();
    let manager = PostgresConnectionManager::new(
        format!(
            "postgres://postgres:postgres@127.0.0.1:{}/postgres",
            node.get_host_port(5432).unwrap()
        ),
        r2d2_postgres::TlsMode::None,
    )
    .unwrap();
    Pool::new(manager).unwrap()
}
