use once_cell::sync::OnceCell;
use rand::distributions::Alphanumeric;
use rand::Rng;
use serde_derive::{Deserialize, Serialize};

pub mod codec;

#[derive(Debug, PartialEq)]
pub enum DbType {
    MySql,
    Pg,
    InMemory,
    Imdb,
    Sqlite,
    TiDB,
}

pub fn rand_string(len: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .collect::<String>()
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct TestData {
    pub first_name: String,
    pub last_name: String,
}

pub fn test<F: std::future::Future>(f: F) -> F::Output {
    static RT: OnceCell<tokio::runtime::Runtime> = OnceCell::new();
    RT.get_or_init(|| {
        tokio::runtime::Builder::new()
            .threaded_scheduler()
            .enable_all()
            .build()
            .expect("Should create a tokio runtime")
    })
    .handle()
    .enter(|| futures::executor::block_on(f))
}
