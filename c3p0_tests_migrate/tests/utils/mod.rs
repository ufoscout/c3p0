use rand::distributions::Alphanumeric;
use rand::Rng;

pub fn rand_string(len: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .map(char::from)
        .take(len)
        .collect::<String>()
}

#[derive(Debug, PartialEq)]
pub enum DbType {
    MySql,
    Pg,
    InMemory,
    Imdb,
    Sqlite,
    TiDB,
}
