#![cfg(feature = "sqlx_sqlite")]

use c3p0::sqlx::sqlx::sqlite::*;
use c3p0::sqlx::sqlx::Row;
use c3p0::sqlx::*;
use c3p0::*;
use maybe_single::nio::{Data, MaybeSingleAsync};
use once_cell::sync::OnceCell;

pub type C3p0Impl = SqlxSqliteC3p0Pool;

//mod tests;
mod tests_json;
mod utils;

pub type MaybeType = (
    C3p0Impl,
    (),
);

async fn init() -> MaybeType {

    let options = SqliteConnectOptions::new();
    let pool = SqlitePool::connect_with(options).await.unwrap();
    let pool = SqlxSqliteC3p0Pool::new(pool);

    (pool, ())
}

pub async fn data(serial: bool) -> Data<'static, MaybeType> {
    static DATA: OnceCell<MaybeSingleAsync<MaybeType>> = OnceCell::new();
    DATA.get_or_init(|| MaybeSingleAsync::new(|| Box::pin(init())))
        .data(serial)
        .await
}

pub mod db_specific {

    use super::*;

    pub fn db_type() -> utils::DbType {
        utils::DbType::Sqlite
    }

    pub fn row_to_string(row: &SqliteRow) -> Result<String, Box<dyn std::error::Error>> {
        let value: String = row.get(0);
        Ok(value)
    }

    pub fn build_insert_query(table_name: &str) -> String {
        format!(r"INSERT INTO {} (name) VALUES (?)", table_name)
    }
}
