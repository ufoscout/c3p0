#[derive(Debug, PartialEq)]
pub enum DbType {
    MySql,
    Pg,
    InMemory,
    Imdb,
    MariaDB,
    Sqlite,
    TiDB,
}
