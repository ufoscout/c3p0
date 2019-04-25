use crate::error::C3p0MigrateError;
use crate::migration::{Migration, Migrations};
use crate::sql_migration::{to_sql_migrations, SqlMigration};
use c3p0::prelude::*;
use log::*;
use serde_derive::{Deserialize, Serialize};
use c3p0::json::codec::DefaultJsonCodec;

pub mod error;
mod md5;
mod sql_migration;
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

    pub fn with_migrations<M: Into<Migrations>>(mut self, migrations: M) -> PgMigrateBuilder {
        self.migrations = migrations.into().migrations;
        self
    }

    pub fn build(self) -> PgMigrate {
        let conf = JsonManagerBuilder::new(self.table.clone())
            .with_schema_name(self.schema.clone())
            .build();

        let repo = C3p0JsonRepository::build(conf);

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
    repo: C3p0JsonRepository<MigrationData, DefaultJsonCodec, JsonManager<'static, MigrationData, DefaultJsonCodec>>,
}

impl PgMigrate {
    pub fn migrate(&self, c3p0: &C3p0) -> Result<(), C3p0MigrateError> {
        {
            let conn = c3p0.connection()?;
            if let Err(err) = self.repo.create_table_if_not_exists(&conn) {
                warn!("Create table process completed with error. This 'COULD' be fine if another process attempted the same operation concurrently. Err: {}", err);
            };
        }

        c3p0.transaction(|conn| {
            Ok(self.start_migration(conn)?)
        }).map_err(C3p0MigrateError::from)

    }

    fn start_migration(&self, conn: &Transaction) -> Result<(), C3p0MigrateError> {
        conn.execute(
            &format!(
                "LOCK TABLE {} IN ACCESS EXCLUSIVE MODE;",
                self.repo.json_manager().qualified_table_name
            ),
            &[],
        )?;

        let migration_history = self.fetch_migrations_history(conn)?;
        let migration_history = PgMigrate::clean_history(migration_history)?;

        for i in 0..self.migrations.len() {
            let migration = &self.migrations[i];

            if migration_history.len() > i {
                let applied_migration = &migration_history[i];

                if applied_migration.data.migration_id.eq(&migration.id) {
                    if applied_migration.data.md5_checksum.eq(&migration.up.md5) {
                        continue;
                    }
                    return Err(C3p0MigrateError::AlteredMigrationSql {
                        message: format!(
                            "Wrong checksum for migration [{}]. Expected [{}], found [{}].",
                            applied_migration.data.migration_id,
                            applied_migration.data.md5_checksum,
                            migration.up.md5
                        ),
                    });
                }
                return Err(C3p0MigrateError::WrongMigrationSet {
                    message: format!(
                        "Wrong migration set! Expected migration [{}], found [{}].",
                        applied_migration.data.migration_id, migration.id
                    ),
                });
            }

            conn.batch_execute(&migration.up.sql)?;

            self.repo.save(
                conn,
                NewModel::new(MigrationData {
                    success: true,
                    md5_checksum: migration.up.md5.clone(),
                    migration_id: migration.id.clone(),
                    migration_type: MigrationType::UP,
                    execution_time_ms: 0,
                    installed_on_epoch_ms: 0,
                }),
            )?;
        }

        Ok(())
    }

    pub fn fetch_migrations_history(
        &self,
        conn: &Transaction,
    ) -> Result<Vec<MigrationModel>, C3p0MigrateError> {
        self.repo.find_all(conn).map_err(C3p0MigrateError::from)
    }

    fn clean_history(
        migrations: Vec<MigrationModel>,
    ) -> Result<Vec<MigrationModel>, C3p0MigrateError> {
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
                        return Err(C3p0MigrateError::CorruptedDbMigrationState {
                            message: "Migration history is not valid!!".to_owned(),
                        });
                    }
                }
            }
        }

        Ok(result)
    }
}
