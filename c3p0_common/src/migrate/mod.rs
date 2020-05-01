use crate::error::C3p0Error;
use crate::json::model::Model;
use crate::migrate::sql_migration::{to_sql_migrations, SqlMigration};
use serde::{Deserialize, Serialize};

pub mod md5;
pub mod migration;
pub mod sql_migration;

pub mod include_dir {
    pub use include_dir::*;
}

pub use migration::{from_embed, from_fs, Migration, Migrations};

pub const C3P0_MIGRATE_TABLE_DEFAULT: &str = "C3P0_MIGRATE_SCHEMA_HISTORY";
pub const C3P0_INIT_MIGRATION_ID: &str = "C3P0_INIT_MIGRATION";

#[derive(Clone, Debug)]
pub struct C3p0MigrateBuilder<C3P0> {
    pub table: String,
    pub schema: Option<String>,
    pub migrations: Vec<SqlMigration>,
    pub c3p0: C3P0,
}

impl<C3P0> C3p0MigrateBuilder<C3P0> {
    pub fn new(c3p0: C3P0) -> Self {
        C3p0MigrateBuilder {
            table: C3P0_MIGRATE_TABLE_DEFAULT.to_owned(),
            schema: None,
            migrations: vec![],
            c3p0,
        }
    }

    pub fn with_schema_name<T: Into<Option<String>>>(
        mut self,
        schema_name: T,
    ) -> C3p0MigrateBuilder<C3P0> {
        self.schema = schema_name.into();
        self
    }

    pub fn with_table_name<T: Into<String>>(mut self, table_name: T) -> C3p0MigrateBuilder<C3P0> {
        self.table = table_name.into();
        self
    }

    pub fn with_migrations<M: Into<Migrations>>(
        mut self,
        migrations: M,
    ) -> C3p0MigrateBuilder<C3P0> {
        self.migrations = to_sql_migrations(migrations.into().migrations);
        self
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
    C3P0INIT,
    UP,
    DOWN,
}

pub fn clean_history(migrations: Vec<MigrationModel>) -> Result<Vec<MigrationModel>, C3p0Error> {
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
                    return Err(C3p0Error::CorruptedDbMigrationState {
                        message: "Migration history is not valid!!".to_owned(),
                    });
                }
            }
            MigrationType::C3P0INIT => {}
        }
    }

    Ok(result)
}

/// Returns whether the migration was already applied
pub fn check_if_migration_already_applied(
    migration_history: &[MigrationModel],
    sql_migration: &SqlMigration,
    check_index: usize,
) -> Result<bool, C3p0Error> {
    if migration_history.len() > check_index {
        let applied_migration = &migration_history[check_index];

        if applied_migration.data.migration_id.eq(&sql_migration.id) {
            if applied_migration
                .data
                .md5_checksum
                .eq(&sql_migration.up.md5)
            {
                return Ok(true);
            }
            return Err(C3p0Error::AlteredMigrationSql {
                message: format!(
                    "Wrong checksum for migration [{}]. Expected [{}], found [{}].",
                    applied_migration.data.migration_id,
                    applied_migration.data.md5_checksum,
                    sql_migration.up.md5
                ),
            });
        }
        return Err(C3p0Error::WrongMigrationSet {
            message: format!(
                "Wrong migration set! Expected migration [{}], found [{}].",
                applied_migration.data.migration_id, sql_migration.id
            ),
        });
    };
    Ok(false)
}

pub fn build_migration_zero() -> MigrationData {
    MigrationData {
        md5_checksum: "".to_owned(),
        migration_id: C3P0_INIT_MIGRATION_ID.to_owned(),
        migration_type: MigrationType::C3P0INIT,
        execution_time_ms: 0,
        installed_on_epoch_ms: 0,
        success: true,
    }
}
