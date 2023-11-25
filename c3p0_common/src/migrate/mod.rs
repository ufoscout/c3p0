use crate::error::C3p0Error;
use crate::json::model::Model;
use crate::migrate::sql_migration::{to_sql_migrations, SqlMigration};
use async_trait::async_trait;
use log::*;
use serde::{Deserialize, Serialize};

pub mod md5;
pub mod migration;
pub mod sql_migration;

pub mod include_dir {
    pub use include_dir::*;
}

use crate::{C3p0Json, C3p0Pool, DefaultJsonCodec, NewModel, SqlConnection};
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

#[async_trait]
pub trait C3p0Migrator: Clone + Send + Sync {
    type Conn: SqlConnection;
    type C3P0: C3p0Pool<Conn = Self::Conn>;
    type C3P0Json: C3p0Json<MigrationData, DefaultJsonCodec, Conn = Self::Conn>;

    fn build_cp30_json(&self, table: String, schema: Option<String>) -> Self::C3P0Json;

    async fn lock_table(
        &self,
        c3p0_json: &Self::C3P0Json,
        conn: &mut Self::Conn,
    ) -> Result<(), C3p0Error>;

    async fn lock_first_migration_row(
        &self,
        c3p0_json: &Self::C3P0Json,
        conn: &mut Self::Conn,
    ) -> Result<(), C3p0Error>;
}

pub struct C3p0Migrate<
    Conn: SqlConnection,
    C3P0: C3p0Pool<Conn = Conn>,
    Migrator: C3p0Migrator<Conn = Conn>,
> {
    table: String,
    schema: Option<String>,
    migrations: Vec<SqlMigration>,
    c3p0: C3P0,
    migrator: Migrator,
}

impl<
        Conn: SqlConnection,
        C3P0: C3p0Pool<Conn = Conn>,
        Migrator: C3p0Migrator<Conn = Conn>,
    > C3p0Migrate<Conn, C3P0, Migrator>
{
    pub fn new(
        table: String,
        schema: Option<String>,
        migrations: Vec<SqlMigration>,
        c3p0: C3P0,
        migrator: Migrator,
    ) -> Self {
        Self {
            table,
            schema,
            migrations,
            c3p0,
            migrator,
        }
    }

    pub async fn migrate(&self) -> Result<(), C3p0Error> {
        let c3p0_json = self
            .migrator
            .build_cp30_json(self.table.clone(), self.schema.clone());

        // Pre Migration
        self.pre_migration(&c3p0_json)
            .await
            .map_err(|err| C3p0Error::MigrationError {
                message: "C3p0Migrate - Failed to execute pre-migration DB preparation."
                    .to_string(),
                source: Box::new(err),
            })?;

        // Start Migration
        self.c3p0
            .transaction(|conn| async {
                self.migrator
                    .lock_first_migration_row(&c3p0_json, conn)
                    .await?;
                Ok(self.start_migration(&c3p0_json, conn).await?)
            })
            .await
            .map_err(|err| C3p0Error::MigrationError {
                message: "C3p0Migrate - Failed to execute DB migration script.".to_string(),
                source: err,
            })
    }

    pub async fn get_migrations_history(
        &self,
        conn: &mut Migrator::Conn,
    ) -> Result<Vec<MigrationModel>, C3p0Error> {
        let c3p0_json = self
            .migrator
            .build_cp30_json(self.table.clone(), self.schema.clone());
        c3p0_json.fetch_all(conn).await
    }

    async fn create_migration_zero(
        &self,
        c3p0_json: &Migrator::C3P0Json,
        conn: &mut Migrator::Conn,
    ) -> Result<(), C3p0Error> {
        let count = c3p0_json.count_all(conn).await?;

        if count == 0 {
            c3p0_json.save(conn, build_migration_zero().into()).await?;
        };

        Ok(())
    }

    async fn pre_migration(&self, c3p0_json: &Migrator::C3P0Json) -> Result<(), C3p0Error> {
        {
            let result = self
                .c3p0
                .transaction(|conn| async { c3p0_json.create_table_if_not_exists(conn).await })
                .await;
            if let Err(err) = result {
                warn!("C3p0Migrate - Create table process completed with error. This 'COULD' be fine if another process attempted the same operation concurrently. Err: {:?}", err);
            };
        }

        // Start Migration
        self.c3p0
            .transaction(|conn| async {
                self.migrator.lock_table(c3p0_json, conn).await?;
                self.create_migration_zero(c3p0_json, conn).await
            })
            .await
    }

    async fn start_migration(
        &self,
        c3p0_json: &Migrator::C3P0Json,
        conn: &mut Migrator::Conn,
    ) -> Result<(), C3p0Error> {
        let migration_history = self.fetch_migrations_history(c3p0_json, conn).await?;
        let migration_history = clean_history(migration_history)?;

        for i in 0..self.migrations.len() {
            let migration = &self.migrations[i];

            if check_if_migration_already_applied(&migration_history, migration, i)? {
                continue;
            }

            conn.batch_execute(&migration.up.sql).await.map_err(|err| {
                C3p0Error::MigrationError {
                    message: format!(
                        "C3p0Migrate - Failed to execute migration with id [{}].",
                        &migration.id
                    ),
                    source: Box::new(err),
                }
            })?;

            c3p0_json
                .save(
                    conn,
                    NewModel::new(MigrationData {
                        success: true,
                        md5_checksum: migration.up.md5.clone(),
                        migration_id: migration.id.clone(),
                        migration_type: MigrationType::UP,
                        execution_time_ms: 0,
                        installed_on_epoch_ms: 0,
                    }),
                )
                .await?;
        }

        Ok(())
    }

    async fn fetch_migrations_history(
        &self,
        c3p0_json: &Migrator::C3P0Json,
        conn: &mut Migrator::Conn,
    ) -> Result<Vec<MigrationModel>, C3p0Error> {
        c3p0_json.fetch_all(conn).await
    }
}

fn clean_history(migrations: Vec<MigrationModel>) -> Result<Vec<MigrationModel>, C3p0Error> {
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
fn check_if_migration_already_applied(
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

fn build_migration_zero() -> MigrationData {
    MigrationData {
        md5_checksum: "".to_owned(),
        migration_id: C3P0_INIT_MIGRATION_ID.to_owned(),
        migration_type: MigrationType::C3P0INIT,
        execution_time_ms: 0,
        installed_on_epoch_ms: 0,
        success: true,
    }
}
