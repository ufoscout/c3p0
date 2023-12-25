#![cfg(feature = "mongodb")]

use c3p0::mongodb::*;
use c3p0::*;
use c3p0_mongodb::mongodb::{bson::oid::ObjectId, Client};
use maybe_single::tokio::{Data, MaybeSingleAsync};
use once_cell::sync::OnceCell;
use rustainers::{
    compose::{
        ComposeContainers, RunnableComposeContainers, RunnableComposeContainersBuilder,
        TemporaryDirectory, TemporaryFile, ToRunnableComposeContainers,
    },
    runner::Runner,
    ExposedPort, PortError, WaitStrategy,
};

pub type C3p0Impl = MongodbC3p0Pool;
pub type Builder = MongodbC3p0JsonBuilder<ObjectId>;

// mod tests;
mod tests_json;
mod utils;

pub type MaybeType = (MongodbC3p0Pool, ComposeContainers<MondodbReplicaSet>);

async fn init() -> MaybeType {
    let image = MondodbReplicaSet::new().await;
    let runner = Runner::auto().unwrap();
    let containers: ComposeContainers<MondodbReplicaSet> =
        runner.compose_start(image).await.unwrap();
    let url = containers.url().await.unwrap();
    let client = Client::with_uri_str(&url).await.unwrap();
    let pool = MongodbC3p0Pool::new(client, "TEST_DB".to_owned());
    (pool, containers)
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

pub struct MondodbReplicaSet {
    temp_dir: TemporaryDirectory,
    port: ExposedPort,
}

impl MondodbReplicaSet {
    async fn new() -> Self {
        let temp_dir = TemporaryDirectory::with_files(
            "componse-mongodb-replica-set",
            [TemporaryFile::builder()
                .with_path("docker-compose.yaml")
                .with_content(
                    r#"
                    version: "3.3"

                    services:
                    
                      mongodb:
                        image: mongo:7
                        container_name: mongodb
                        ports:
                          - '27017:27017'
                        command: mongod --replSet rs0
                        # the healthcheck is used to initialize the replica set
                        healthcheck:
                          test: |
                            mongosh --eval "try { rs.status().ok } catch (e) { rs.initiate({ _id: 'rs0', members: [{ _id: 0, host: 'localhost:27017' }] }).ok }"
                          start_period: 0s
                          interval: 500ms
                          timeout: 5s
                          retries: 5
               
                    "#,
                )
                .build()],
        )
        .await
        .expect("Should create temp_dir");

        let port = ExposedPort::new(27017);
        Self { temp_dir, port }
    }

    async fn url(&self) -> Result<String, PortError> {
        let port = self.port.host_port().await?;
        Ok(format!("mongodb://127.0.0.1:{port}/"))
    }
}

impl ToRunnableComposeContainers for MondodbReplicaSet {
    type AsPath = TemporaryDirectory;

    fn to_runnable(
        &self,
        builder: RunnableComposeContainersBuilder<Self::AsPath>,
    ) -> RunnableComposeContainers<Self::AsPath> {
        builder
            .with_compose_path(self.temp_dir.clone())
            .with_wait_strategies([("mongodb", WaitStrategy::HealthCheck)])
            .with_port_mappings([("mongodb", self.port.clone())])
            .build()
    }
}
