use crate::migration::{Migration, SqlMigration};
use c3p0_pg::{C3p0Repository, ConfigBuilder, C3p0, Model, NewModel};
use postgres::Connection;
use serde_derive::{Deserialize, Serialize};

mod md5;
pub mod migration;

pub const C3P0_MIGRATE_TABLE_DEFAULT: &str = "C3P0_MIGRATE_SCHEMA_HISTORY";

#[derive(Clone, Debug)]
pub struct PgMigrateBuilder {
    table: String,
    schema: Option<String>,
    migrations: Vec<Migration>,
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

        let repo = C3p0Repository::build(conf);

        PgMigrate {
            table: self.table,
            schema: self.schema,
            migrations: self.migrations.into_iter().map(|migration| SqlMigration::new(migration)).collect(),
            repo,
        }
    }
}

pub type MigrationModel = Model<MigrationData>;

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct MigrationData {
    pub migration_id: String,
    pub migration_type: MigrationType,
    pub md5_checksum: String,
    pub installed_on_epoch_ms: u64,
    pub execution_time_ms: u64,
    pub success: bool,
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub enum MigrationType {
    UP,
    DOWN,
}

#[derive(Clone)]
pub struct PgMigrate {
    table: String,
    schema: Option<String>,
    migrations: Vec<SqlMigration>,
    repo: C3p0Repository<MigrationData>,
}

impl PgMigrate {

    pub fn migrate(&self, conn: &Connection) {
        let tx = conn.transaction().unwrap();

        self.repo.create_table_if_not_exists(tx.connection()).unwrap();

        for migration in &self.migrations {
            tx.batch_execute(&migration.up.sql).unwrap();

            self.repo.save(tx.connection(), NewModel::new(MigrationData{
                success: true,
                md5_checksum: migration.up.md5.clone(),
                migration_id: migration.id.clone(),
                migration_type: MigrationType::UP,
                execution_time_ms: 0,
                installed_on_epoch_ms: 0,
            })).unwrap();

        }

        tx.commit().unwrap();
    }

    pub fn migrations_status(&self, conn: &Connection) -> Vec<MigrationModel> {
        self.repo.find_all(conn).unwrap()
    }

}