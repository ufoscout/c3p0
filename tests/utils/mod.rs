use std::sync::OnceLock;

#[derive(Debug, PartialEq)]
pub enum DbType {
    MySql,
    Pg,
    InMemory,
    Imdb,
    Sqlite,
    TiDB,
}

pub fn run_test<F: std::future::Future>(f: F) -> F::Output {
    static RT: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
    RT.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Should create a tokio runtime")
    })
    .block_on(f)
}
