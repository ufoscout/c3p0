#![cfg(feature = "sqlx_postgres")]

use c3p0::sqlx::sqlx::postgres::*;
use c3p0::sqlx::sqlx::Row;
use c3p0::sqlx::*;
use c3p0::*;
use maybe_single::nio::{Data, MaybeSingleAsync};
use once_cell::sync::OnceCell;
use testcontainers::postgres::Postgres;
use testcontainers::testcontainers::clients::Cli;
use testcontainers::testcontainers::Container;

pub type C3p0Impl = SqlxPgC3p0Pool;

//mod tests;
mod tests_json;
mod utils;

pub type MaybeType = (C3p0Impl, Container<'static, Postgres>);

async fn init() -> MaybeType {
    static DOCKER: OnceCell<Cli> = OnceCell::new();
    let node = DOCKER
        .get_or_init(Cli::default)
        .run(Postgres::default());

    let options = PgConnectOptions::new()
        .username("postgres")
        .password("postgres")
        .database("postgres")
        .host("127.0.0.1")
        .port(node.get_host_port_ipv4(5432));

    let pool = PgPool::connect_with(options).await.unwrap();

    let pool = SqlxPgC3p0Pool::new(pool);

    (pool, node)
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
        utils::DbType::Pg
    }

    pub fn row_to_string(row: &PgRow) -> Result<String, Box<dyn std::error::Error>> {
        let value: String = row.get(0);
        Ok(value)
    }

    pub fn build_insert_query(table_name: &str) -> String {
        format!(r"INSERT INTO {} (name) VALUES ($1)", table_name)
    }
}
