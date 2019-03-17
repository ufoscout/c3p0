use lazy_static::lazy_static;
use r2d2::{Pool, PooledConnection};
use r2d2_postgres::PostgresConnectionManager;
use serde_derive::{Deserialize, Serialize};
use std::sync::Mutex;
use testcontainers::*;
use std::sync::atomic::{AtomicUsize, Ordering::SeqCst};

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

struct TestSingleton<T: Drop> {
    data: Mutex<Option<T>>,
    init: fn() -> T,
    callers: Mutex<AtomicUsize>
}

impl <T: Drop> TestSingleton<T> {

    pub fn new(init: fn() -> T) -> Self {
        TestSingleton {
            data: Mutex::new(None),
            init,
            callers: Mutex::new(AtomicUsize::new(0))
        }
    }

    pub fn get(&self, callback: fn(data: T)) {
        {
            let lock= self.callers.lock().unwrap();
            let callers = lock.load(SeqCst) + 1;
            lock.store(callers, SeqCst);
        }
        {
            let lock_ = self.data.lock().unwrap();

        }
        {
            let lock= self.callers.lock().unwrap();
            let callers = lock.load(SeqCst) - 1;
            lock.store(callers, SeqCst);

            if callers == 0 {
                let mut data = self.data.lock().unwrap();
                *data = None;
            }
        }

    }
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
