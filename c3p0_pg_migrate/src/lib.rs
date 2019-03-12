use crate::migration::Migration;
use c3p0_pg::{SimpleRepository, ConfigBuilder};
use postgres::Connection;
use serde_derive::{Deserialize, Serialize};

mod md5;
pub mod migration;

const C3P0_MIGRATE_TABLE_DEFAULT: &str = "C3P0_MIGRATE_SCHEMA_HISTORY";

#[derive(Clone, Debug)]
pub struct PgMigrateBuilder {
    table: String,
    schema: Option<String>,
    migrations: Vec<Migration>
}

impl PgMigrateBuilder {

    pub fn new() -> Self {
        PgMigrateBuilder {
            table: C3P0_MIGRATE_TABLE_DEFAULT.to_owned(),
            schema: None,
            migrations: vec![],
        }
    }

    pub fn with_schema_name<T: Into<Option<String>>>(mut self, schema_name: T) -> PgMigrateBuilder {
        self.schema = schema_name.into();
        self
    }

    pub fn with_table_name<T: Into<String>>(mut self, table_name: T) -> PgMigrateBuilder {
        self.table = table_name.into();
        self
    }

    pub fn with_migrations(mut self, migrations: Vec<Migration>) -> PgMigrateBuilder {
        self.migrations = migrations;
        self
    }

    pub fn build(self) -> PgMigrate {

        let conf = ConfigBuilder::new(self.table.clone())
            .with_schema_name(self.schema.clone())
            .build();

        let repo = SimpleRepository::build(conf);

        PgMigrate {
            table: self.table,
            schema: self.schema,
            migrations: self.migrations,
            repo
        }
    }
}

#[derive(Clone)]
pub struct PgMigrate {
    table: String,
    schema: Option<String>,
    migrations: Vec<Migration>,
    repo: SimpleRepository<MigrationData>
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
struct MigrationData {
    migration_id: String,
    migration_type: MigrationType,
    md5_checksum: String,
    installed_on_epoch_ms: u64,
    execution_time_ms: u64,
    success: bool,
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
enum MigrationType {
    UP,
    DOWN,
}
