use crate::blocking::*;
use log::*;

use crate::migrate::sql_migration::SqlMigration;
use crate::migrate::{build_migration_zero, check_if_migration_already_applied, clean_history};

#[derive(Clone)]
pub struct C3p0Migrate<
    CONN: SqlConnection,
    C3P0: C3p0Pool<CONN = CONN>,
    MIGRATOR: Migrator<CONN = CONN>,
> {
    table: String,
    schema: Option<String>,
    migrations: Vec<SqlMigration>,
    c3p0: C3P0,
    migrator: MIGRATOR,
}

impl<CONN: SqlConnection, C3P0: C3p0Pool<CONN = CONN>, MIGRATOR: Migrator<CONN = CONN>>
    C3p0Migrate<CONN, C3P0, MIGRATOR>
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

    pub fn migrate(&self) -> Result<(), C3p0Error> {
        let c3p0_json = self
            .migrator
            .build_cp30_json(self.table.clone(), self.schema.clone());

        // Pre Migration
        self.pre_migration(&c3p0_json)
            .map_err(|err| C3p0Error::MigrationError {
                message: "C3p0Migrate - Failed to execute pre-migration DB preparation."
                    .to_string(),
                cause: Box::new(err),
            })?;

        // Start Migration
        self.c3p0
            .transaction(|conn| {
                self.migrator.lock_first_migration_row(&c3p0_json, conn)?;
                Ok(self.start_migration(&c3p0_json, conn)?)
            })
            .map_err(|err| C3p0Error::MigrationError {
                message: "C3p0Migrate - Failed to execute DB migration script.".to_string(),
                cause: err,
            })
    }

    pub fn get_migrations_history(
        &self,
        conn: &mut MIGRATOR::CONN,
    ) -> Result<Vec<MigrationModel>, C3p0Error> {
        let c3p0_json = self
            .migrator
            .build_cp30_json(self.table.clone(), self.schema.clone());
        c3p0_json.fetch_all(conn)
    }

    fn create_migration_zero(
        &self,
        c3p0_json: &MIGRATOR::C3P0JSON,
        conn: &mut MIGRATOR::CONN,
    ) -> Result<(), C3p0Error> {
        let count = c3p0_json.count_all(conn)?;

        if count == 0 {
            c3p0_json.save(conn, build_migration_zero().into())?;
        };

        Ok(())
    }

    fn pre_migration(&self, c3p0_json: &MIGRATOR::C3P0JSON) -> Result<(), C3p0Error> {
        {
            let result = self
                .c3p0
                .transaction(|conn| c3p0_json.create_table_if_not_exists(conn));
            if let Err(err) = result {
                warn!("C3p0Migrate - Create table process completed with error. This 'COULD' be fine if another process attempted the same operation concurrently. Err: {}", err);
            };
        }

        // Start Migration
        self.c3p0.transaction(|conn| {
            self.migrator.lock_table(&c3p0_json, conn)?;
            Ok(self.create_migration_zero(&c3p0_json, conn)?)
        })
    }

    fn start_migration(
        &self,
        c3p0_json: &MIGRATOR::C3P0JSON,
        conn: &mut MIGRATOR::CONN,
    ) -> Result<(), C3p0Error> {
        let migration_history = self.fetch_migrations_history(c3p0_json, conn)?;
        let migration_history = clean_history(migration_history)?;

        for i in 0..self.migrations.len() {
            let migration = &self.migrations[i];

            if check_if_migration_already_applied(&migration_history, &migration, i)? {
                continue;
            }

            conn.batch_execute(&migration.up.sql)
                .map_err(|err| C3p0Error::MigrationError {
                    message: format!(
                        "C3p0Migrate - Failed to execute migration with id [{}].",
                        &migration.id
                    ),
                    cause: Box::new(err),
                })?;

            c3p0_json.save(
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

    fn fetch_migrations_history(
        &self,
        c3p0_json: &MIGRATOR::C3P0JSON,
        conn: &mut MIGRATOR::CONN,
    ) -> Result<Vec<MigrationModel>, C3p0Error> {
        c3p0_json.fetch_all(conn)
    }
}

pub trait Migrator: Clone {
    type CONN: SqlConnection;
    type C3P0: C3p0Pool<CONN = Self::CONN>;
    type C3P0JSON: C3p0Json<MigrationData, DefaultJsonCodec, CONN = Self::CONN>;

    fn build_cp30_json(&self, table: String, schema: Option<String>) -> Self::C3P0JSON;

    fn lock_table(
        &self,
        c3p0_json: &Self::C3P0JSON,
        conn: &mut Self::CONN,
    ) -> Result<(), C3p0Error>;

    fn lock_first_migration_row(
        &self,
        c3p0_json: &Self::C3P0JSON,
        conn: &mut Self::CONN,
    ) -> Result<(), C3p0Error>;
}
