use rand::distributions::Alphanumeric;
use rand::Rng;
use serde_derive::{Deserialize, Serialize};

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
