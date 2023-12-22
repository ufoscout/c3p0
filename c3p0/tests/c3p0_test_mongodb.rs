#![cfg(feature = "mongodb")]

use c3p0::mongodb::*;
use c3p0::*;
use c3p0_mongodb::mongodb::Client;
use maybe_single::tokio::{Data, MaybeSingleAsync};
use once_cell::sync::OnceCell;
use testcontainers::{
    mongo::Mongo,
    testcontainers::{clients::Cli, Container},
};


pub type C3p0Impl = MongodbC3p0Pool;

// mod tests;
mod tests_json;
mod utils;

pub type MaybeType = (C3p0Impl, Container<'static, Mongo>);

async fn init() -> MaybeType {
    static DOCKER: OnceCell<Cli> = OnceCell::new();
    let node = DOCKER.get_or_init(Cli::default).run(Mongo::default());

    let host_port = node.get_host_port_ipv4(27017);
    let url = format!("mongodb://127.0.0.1:{host_port}/");

    let client = Client::with_uri_str(&url).await.unwrap();

    let pool = MongodbC3p0Pool::new(client, "TEST_DB".to_owned());

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
        utils::DbType::Mongodb
    }

    // pub fn row_to_string(row: &Row) -> Result<String, Box<dyn std::error::Error>> {
    //     let value: String = row.get(0);
    //     Ok(value)
    // }

    // pub fn build_insert_query(table_name: &str) -> String {
    //     format!(r"INSERT INTO {} (name) VALUES ($1)", table_name)
    // }
}
