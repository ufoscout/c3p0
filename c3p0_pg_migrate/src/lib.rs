use crate::migration::{to_sql_migrations, Migration, SqlMigration};
use c3p0_pg::{C3p0, C3p0Repository, ConfigBuilder, Model, NewModel};
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

impl Default for PgMigrateBuilder {
    fn default() -> Self {
        PgMigrateBuilder {
            table: C3P0_MIGRATE_TABLE_DEFAULT.to_owned(),
            schema: None,
            migrations: vec![],
        }
    }
}

impl PgMigrateBuilder {
    pub fn new() -> Self {
        Default::default()
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
            migrations: to_sql_migrations(self.migrations),
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

        self.repo
            .create_table_if_not_exists(tx.connection())
            .unwrap();

        let migration_history = self.fetch_migrations_history(conn);
        let migration_history = PgMigrate::clean_history(migration_history);

        for i in 0..self.migrations.len() {
            let migration = &self.migrations[i];

            if migration_history.len() > i {
                let applied_migration = &migration_history[i];

                if applied_migration.data.migration_id.eq(&migration.id) {
                    if applied_migration.data.md5_checksum.eq(&migration.up.md5) {
                        continue;
                    }
                    panic!(
                        "Wrong checksum for migration [{}]. Expected [{}], found [{}].",
                        applied_migration.data.migration_id,
                        applied_migration.data.md5_checksum,
                        migration.up.md5
                    );
                }
                panic!(
                    "Wrong migration set! Expected migration [{}], found [{}].",
                    applied_migration.data.migration_id, migration.id
                );
            }

            tx.batch_execute(&migration.up.sql).unwrap();

            self.repo
                .save(
                    tx.connection(),
                    NewModel::new(MigrationData {
                        success: true,
                        md5_checksum: migration.up.md5.clone(),
                        migration_id: migration.id.clone(),
                        migration_type: MigrationType::UP,
                        execution_time_ms: 0,
                        installed_on_epoch_ms: 0,
                    }),
                )
                .unwrap();
        }

        tx.commit().unwrap();
    }

    pub fn fetch_migrations_history(&self, conn: &Connection) -> Vec<MigrationModel> {
        self.repo.find_all(conn).unwrap()
    }

    fn clean_history(migrations: Vec<MigrationModel>) -> Vec<MigrationModel> {
        let mut result = vec![];

        for migration in migrations {
            match migration.data.migration_type {
                MigrationType::UP => {
                    result.push(migration);
                }
                MigrationType::DOWN => {
                    let last = result.remove(result.len() - 1);
                    if !migration.data.migration_id.eq(&last.data.migration_id)
                        || !last.data.migration_type.eq(&MigrationType::UP)
                    {
                        panic!("migration history is not valid!!");
                    }
                }
            }
        }

        result
    }
}
