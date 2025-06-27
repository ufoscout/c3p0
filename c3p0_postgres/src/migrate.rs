use crate::*;
use c3p0_common::*;
use log::warn;

pub trait PgC3p0MigrateBuilder {
    // WARN: This generic implementation is disabled and replaced by the one below,
    // this is because it causes compilation errors on the `should_handle_parallel_executions` test
    // I am not sure why this is the case
    //
    // fn build(self) -> C3p0Migrate<PgMigrator>;

    fn build(self) -> PgC3p0Migrate;
}

impl PgC3p0MigrateBuilder for C3p0MigrateBuilder<PgC3p0Pool> {
    // fn build(self) -> C3p0Migrate<PgMigrator> {
    //     C3p0Migrate::new(
    //         self.migrations,
    //         self.c3p0,
    //         PgMigrator {
    //             c3p0_json: PgC3p0JsonBuilder::<u64, i64>::new(self.table)
    //             .with_schema_name(self.schema)
    //             .build(),
    //         },
    //     )
    // }

    fn build(self) -> PgC3p0Migrate {
        PgC3p0Migrate::new(
            self.migrations,
            self.c3p0,
            PgMigrator {
                c3p0_json: PgC3p0JsonBuilder::<u64, i64>::new(self.table)
                    .with_schema_name(self.schema)
                    .build(),
            },
        )
    }
}

#[derive(Clone)]
pub struct PgMigrator {
    c3p0_json: PgC3p0Json<u64, i64, MigrationData, DefaultJsonCodec>,
}

impl C3p0Migrator for PgMigrator {
    type C3P0 = PgC3p0Pool;
    type C3P0Json = PgC3p0Json<u64, i64, MigrationData, DefaultJsonCodec>;

    fn cp30_json(&self) -> &Self::C3P0Json {
        &self.c3p0_json
    }

    async fn batch_execute(
        &self,
        sql: &str,
        conn: &mut <<Self as C3p0Migrator>::C3P0 as C3p0Pool>::Tx<'_>,
    ) -> Result<(), C3p0Error> {
        conn.batch_execute(sql).await
    }

    async fn lock_table(
        &self,
        conn: &mut <<Self as C3p0Migrator>::C3P0 as C3p0Pool>::Tx<'_>,
    ) -> Result<(), C3p0Error> {
        conn.batch_execute(&format!(
            "LOCK TABLE {} IN ACCESS EXCLUSIVE MODE",
            self.c3p0_json.queries().qualified_table_name
        ))
        .await
    }

    async fn lock_first_migration_row(
        &self,
        conn: &mut <<Self as C3p0Migrator>::C3P0 as C3p0Pool>::Tx<'_>,
    ) -> Result<(), C3p0Error> {
        let lock_sql = format!(
            r#"select * from {} where {}->>'migration_id' = $1 FOR UPDATE"#,
            self.c3p0_json.queries().qualified_table_name,
            self.c3p0_json.queries().data_field_name
        );
        conn.fetch_one(&lock_sql, &[&C3P0_INIT_MIGRATION_ID], |_| Ok(()))
            .await
    }
}

// This struct is a one-to-one mapping of C3p0Migrate struct but it does not uses generics.
// I am forced to do this because of the compilation errors on the `should_handle_parallel_executions` test
// caused by the generic implementation
pub struct PgC3p0Migrate {
    migrations: Vec<SqlMigration>,
    c3p0: PgC3p0Pool,
    migrator: PgMigrator,
}

impl PgC3p0Migrate {
    pub fn new(migrations: Vec<SqlMigration>, c3p0: PgC3p0Pool, migrator: PgMigrator) -> Self {
        Self {
            migrations,
            c3p0,
            migrator,
        }
    }

    pub async fn migrate(&self) -> Result<(), C3p0Error> {
        // Pre Migration
        self.pre_migration()
            .await
            .map_err(|err| C3p0Error::MigrationError {
                cause: "C3p0Migrate - Failed to execute pre-migration DB preparation.".to_string(),
                source: Box::new(err),
            })?;

        // Start Migration
        self.c3p0
            .transaction(async |conn| {
                self.migrator.lock_first_migration_row(conn).await?;
                Ok(self.start_migration(conn).await?)
            })
            .await
            .map_err(|err| C3p0Error::MigrationError {
                cause: "C3p0Migrate - Failed to execute DB migration script.".to_string(),
                source: err,
            })
    }

    pub async fn get_migrations_history(
        &self,
        conn: &mut PgTx<'_>,
    ) -> Result<Vec<MigrationModel>, C3p0Error> {
        self.migrator.cp30_json().fetch_all(conn).await
    }

    async fn create_migration_zero(&self, conn: &mut PgTx<'_>) -> Result<(), C3p0Error> {
        let c3p0_json = self.migrator.cp30_json();
        let count = c3p0_json.count_all(conn).await?;
        if count == 0 {
            c3p0_json.save(conn, build_migration_zero().into()).await?;
        };
        Ok(())
    }

    async fn pre_migration(&self) -> Result<(), C3p0Error> {
        {
            let result = self
                .c3p0
                .transaction(async |conn| {
                    self.migrator
                        .cp30_json()
                        .create_table_if_not_exists(conn)
                        .await
                })
                .await;
            if let Err(err) = result {
                warn!(
                    "C3p0Migrate - Create table process completed with error. This 'COULD' be fine if another process attempted the same operation concurrently. Err: {err:?}",
                );
            };
        }

        // Start Migration
        self.c3p0
            .transaction(async |conn| {
                self.migrator.lock_table(conn).await?;
                self.create_migration_zero(conn).await
            })
            .await
    }

    async fn start_migration(&self, conn: &mut PgTx<'_>) -> Result<(), C3p0Error> {
        let migration_history = self.fetch_migrations_history(conn).await?;
        let migration_history = clean_history(migration_history)?;

        for i in 0..self.migrations.len() {
            let migration = &self.migrations[i];

            if check_if_migration_already_applied(&migration_history, migration, i)? {
                continue;
            }

            self.migrator
                .batch_execute(&migration.up.sql, conn)
                .await
                .map_err(|err| C3p0Error::MigrationError {
                    cause: format!(
                        "C3p0Migrate - Failed to execute migration with id [{}].",
                        &migration.id
                    ),
                    source: Box::new(err),
                })?;

            self.migrator
                .cp30_json()
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
        conn: &mut PgTx<'_>,
    ) -> Result<Vec<MigrationModel>, C3p0Error> {
        self.migrator.cp30_json().fetch_all(conn).await
    }
}
