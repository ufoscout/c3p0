#![cfg(feature = "sqlite")]

use std::sync::OnceLock;

use c3p0::*;
use maybe_once::tokio::{Data, MaybeOnceAsync};
use ::sqlx::{sqlite::SqliteConnectOptions, Row, Sqlite};

pub type C3p0Impl = SqliteC3p0Pool;

mod tests;
mod utils;

pub type MaybeType = (C3p0Impl, ());

async fn init() -> MaybeType {
    let options = SqliteConnectOptions::new();

    let pool: c3p0::sqlx::Pool<Sqlite> = c3p0::sqlx::pool::PoolOptions::new()
        .max_lifetime(None)
        .idle_timeout(None)
        .max_connections(1)
        .connect_with(options)
        .await
        .unwrap();

    //let pool = SqlitePool::connect_with(options).await.unwrap();

    let pool = SqliteC3p0Pool::new(pool);

    (pool, ())
}

pub async fn data(serial: bool) -> Data<'static, MaybeType> {
    static DATA: OnceLock<MaybeOnceAsync<MaybeType>> = OnceLock::new();
    DATA.get_or_init(|| MaybeOnceAsync::new(|| Box::pin(init())))
        .data(serial)
        .await
}

pub mod db_specific {

    use ::sqlx::sqlite::SqliteRow;

    use super::*;

    pub fn db_type() -> utils::DbType {
        utils::DbType::Sqlite
    }

    pub fn row_to_string(row: &SqliteRow) -> Result<String, Box<dyn std::error::Error>> {
        let value: String = row.get(0);
        Ok(value)
    }

    pub fn build_insert_query(table_name: &str) -> String {
        format!(r"INSERT INTO {table_name} (name) VALUES (?)")
    }
}
