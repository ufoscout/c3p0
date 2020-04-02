use c3p0_common::error::C3p0Error;
use c3p0_common::json::codec::DefaultJsonCodec;
use c3p0_common::json::model::NewModel;
use c3p0_common::migrate::sql_migration::SqlMigration;
use log::*;

use crate::json::C3p0JsonAsync;
use crate::pool::{C3p0PoolAsync, SqlConnectionAsync};
use async_trait::async_trait;
use c3p0_common::{
    build_migration_zero, check_if_migration_already_applied, clean_history, MigrationData,
    MigrationModel, MigrationType,
};

#[async_trait]
pub trait MigratorAsync: Clone + Send + Sync {
    type CONN: SqlConnectionAsync;
    type C3P0: C3p0PoolAsync<CONN = Self::CONN>;
    type C3P0JSON: C3p0JsonAsync<MigrationData, DefaultJsonCodec, CONN = Self::CONN>;

    fn build_cp30_json(&self, table: String, schema: Option<String>) -> Self::C3P0JSON;

    async fn lock_table(
        &self,
        c3p0_json: &Self::C3P0JSON,
        conn: &mut Self::CONN,
    ) -> Result<(), C3p0Error>;

    async fn lock_first_migration_row(
        &self,
        c3p0_json: &Self::C3P0JSON,
        conn: &mut Self::CONN,
    ) -> Result<(), C3p0Error>;
}

pub struct C3p0MigrateAsync<
    CONN: SqlConnectionAsync,
    C3P0: C3p0PoolAsync<CONN = CONN>,
    MIGRATOR: MigratorAsync<CONN = CONN>,
> {
    table: String,
    schema: Option<String>,
    migrations: Vec<SqlMigration>,
    c3p0: C3P0,
    migrator: MIGRATOR,
}

impl<
        CONN: SqlConnectionAsync,
        C3P0: C3p0PoolAsync<CONN = CONN>,
        MIGRATOR: MigratorAsync<CONN = CONN> + Sync,
    > C3p0MigrateAsync<CONN, C3P0, MIGRATOR>
{
    pub fn new(
        table: String,
        schema: Option<String>,
        migrations: Vec<SqlMigration>,
        c3p0: C3P0,
        migrator: MIGRATOR,
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
                cause: Box::new(err),
            })?;

        // Start Migration
        self.c3p0
            .transaction(|mut conn| async move {
                let conn = &mut conn;
                self.migrator
                    .lock_first_migration_row(&c3p0_json, conn)
                    .await?;
                Ok(self.start_migration(&c3p0_json, conn).await?)
            })
            .await
            .map_err(|err| C3p0Error::MigrationError {
                message: "C3p0Migrate - Failed to execute DB migration script.".to_string(),
                cause: err,
            })
    }

    pub async fn get_migrations_history(
        &self,
        conn: &mut MIGRATOR::CONN,
    ) -> Result<Vec<MigrationModel>, C3p0Error> {
        let c3p0_json = self
            .migrator
            .build_cp30_json(self.table.clone(), self.schema.clone());
        c3p0_json.fetch_all(conn).await
    }

    async fn create_migration_zero(
        &self,
        c3p0_json: &MIGRATOR::C3P0JSON,
        conn: &mut MIGRATOR::CONN,
    ) -> Result<(), C3p0Error> {
        let count = c3p0_json.count_all(conn).await?;

        if count == 0 {
            c3p0_json.save(conn, build_migration_zero().into()).await?;
        };

        Ok(())
    }

    async fn pre_migration(&self, c3p0_json: &MIGRATOR::C3P0JSON) -> Result<(), C3p0Error> {
        {
            let result = self
                .c3p0
                .transaction(|mut conn| async move {
                    c3p0_json.create_table_if_not_exists(&mut conn).await
                })
                .await;
            if let Err(err) = result {
                warn!("C3p0Migrate - Create table process completed with error. This 'COULD' be fine if another process attempted the same operation concurrently. Err: {}", err);
            };
        }

        // Start Migration
        self.c3p0
            .transaction(|mut conn| async move {
                self.migrator.lock_table(c3p0_json, &mut conn).await?;
                Ok(self.create_migration_zero(c3p0_json, &mut conn).await?)
            })
            .await
    }

    async fn start_migration(
        &self,
        c3p0_json: &MIGRATOR::C3P0JSON,
        conn: &mut MIGRATOR::CONN,
    ) -> Result<(), C3p0Error> {
        let migration_history = self.fetch_migrations_history(c3p0_json, conn).await?;
        let migration_history = clean_history(migration_history)?;

        for i in 0..self.migrations.len() {
            let migration = &self.migrations[i];

            if check_if_migration_already_applied(&migration_history, &migration, i)? {
                continue;
            }

            conn.batch_execute(&migration.up.sql).await.map_err(|err| {
                C3p0Error::MigrationError {
                    message: format!(
                        "C3p0Migrate - Failed to execute migration with id [{}].",
                        &migration.id
                    ),
                    cause: Box::new(err),
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
        c3p0_json: &MIGRATOR::C3P0JSON,
        conn: &mut MIGRATOR::CONN,
    ) -> Result<Vec<MigrationModel>, C3p0Error> {
        c3p0_json.fetch_all(conn).await
    }
}
