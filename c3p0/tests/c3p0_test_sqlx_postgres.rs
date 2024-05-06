#![cfg(feature = "sqlx_postgres")]

use std::sync::Arc;

use c3p0::sqlx::sqlx::postgres::*;
use c3p0::sqlx::sqlx::Row;
use c3p0::sqlx::*;
use c3p0::*;
use maybe_single::tokio::{Data, MaybeSingleAsync};
use once_cell::sync::OnceCell;
use testcontainers::postgres::Postgres;
use testcontainers::testcontainers::runners::AsyncRunner;
use testcontainers::testcontainers::ContainerAsync;

pub type C3p0Impl = SqlxPgC3p0Pool;
pub type Builder = SqlxPgC3p0JsonBuilder<u64>;
pub type UuidBuilder = SqlxPgC3p0JsonBuilder<uuid::Uuid>;

pub fn new_uuid_builder(table_name: &str) -> UuidBuilder {
    SqlxPgC3p0JsonBuilder::new(table_name).with_id_generator(Arc::new(PostgresUuidIdGenerator {}))
}

//mod tests;
mod tests_json;
mod utils;

pub type MaybeType = (C3p0Impl, ContainerAsync<Postgres>);

async fn init() -> MaybeType {

    let node = Postgres::default().start().await;

    let options = PgConnectOptions::new()
        .username("postgres")
        .password("postgres")
        .database("postgres")
        .host("127.0.0.1")
        .port(node.get_host_port_ipv4(5432).await);

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
