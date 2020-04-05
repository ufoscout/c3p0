#![cfg(feature = "pg_blocking")]

use c3p0::blocking::*;
pub use c3p0::pg::blocking::postgres::{row::Row, NoTls};
use c3p0::pg::blocking::r2d2::{Pool, PostgresConnectionManager};
use c3p0::pg::blocking::*;
use maybe_single::{Data, MaybeSingle};
use once_cell::sync::OnceCell;
use testcontainers::*;

pub type C3p0Impl = PgC3p0Pool;

mod tests_blocking;
mod tests_blocking_json;
pub mod utils;

pub type MaybeType = (
    C3p0Impl,
    Container<'static, clients::Cli, images::postgres::Postgres>,
);

fn init() -> MaybeType {
    static DOCKER: OnceCell<clients::Cli> = OnceCell::new();
    let node = DOCKER
        .get_or_init(|| clients::Cli::default())
        .run(images::postgres::Postgres::default());

    let manager = PostgresConnectionManager::new(
        format!(
            "postgres://postgres:postgres@127.0.0.1:{}/postgres",
            node.get_host_port(5432).unwrap()
        )
        .parse()
        .unwrap(),
        Box::new(move |config| config.connect(NoTls)),
    );

    let pool = Pool::builder().min_idle(Some(10)).build(manager).unwrap();

    let pool = PgC3p0Pool::new(pool);

    (pool, node)
}

pub fn data(serial: bool) -> Data<'static, MaybeType> {
    static DATA: OnceCell<MaybeSingle<MaybeType>> = OnceCell::new();
    DATA.get_or_init(|| MaybeSingle::new(|| init()))
        .data(serial)
}

pub mod db_specific {

    use super::*;

    pub fn db_type() -> utils::DbType {
        utils::DbType::Pg
    }

    pub fn row_to_string(row: &Row) -> Result<String, Box<dyn std::error::Error>> {
        let value: String = row.get(0);
        Ok(value)
    }

    pub fn build_insert_query(table_name: &str) -> String {
        format!(r"INSERT INTO {} (name) VALUES ($1)", table_name)
    }
}