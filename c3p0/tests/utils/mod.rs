use std::sync::OnceLock;

use rand::distributions::Alphanumeric;
use rand::Rng;
use serde::{Deserialize, Serialize};

pub mod codec;

#[derive(Debug, PartialEq)]
pub enum DbType {
    MySql,
    Pg,
    InMemory,
    Imdb,
    Sqlite,
    TiDB,
    Mongodb,
}

pub fn rand_string(len: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .map(char::from)
        .take(len)
        .collect::<String>()
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct TestData {
    pub first_name: String,
    pub last_name: String,
}

pub fn test<F: std::future::Future>(f: F) -> F::Output {
    static RT: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
    RT.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Should create a tokio runtime")
    })
    .block_on(f)
}
