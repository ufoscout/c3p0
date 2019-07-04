use crate::migration::{Migration, Migrations};
use crate::sql_migration::{to_sql_migrations, SqlMigration};
use c3p0::json::codec::DefaultJsonCodec;
use c3p0::json::JsonManagerBase;
use c3p0_json::*;
use log::*;
use serde_derive::{Deserialize, Serialize};

mod md5;
pub mod migration;
mod sql_migration;

pub const C3P0_MIGRATE_TABLE_DEFAULT: &str = "C3P0_MIGRATE_SCHEMA_HISTORY";

#[derive(Clone, Debug)]
pub struct C3p0MigrateBuilder {
    table: String,
    schema: Option<String>,
    migrations: Vec<Migration>,
}

impl Default for C3p0MigrateBuilder {
    fn default() -> Self {
        C3p0MigrateBuilder {
            table: C3P0_MIGRATE_TABLE_DEFAULT.to_owned(),
            schema: None,
            migrations: vec![],
        }
    }
}

impl C3p0MigrateBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_schema_name<T: Into<Option<String>>>(
        mut self,
        schema_name: T,
    ) -> C3p0MigrateBuilder {
        self.schema = schema_name.into();
        self
    }

    pub fn with_table_name<T: Into<String>>(mut self, table_name: T) -> C3p0MigrateBuilder {
        self.table = table_name.into();
        self
    }

    pub fn with_migrations<M: Into<Migrations>>(mut self, migrations: M) -> C3p0MigrateBuilder {
        self.migrations = migrations.into().migrations;
        self
    }

    pub fn build(self) -> C3p0Migrate {
        let conf = JsonManagerBuilder::new(self.table.clone())
            .with_schema_name(self.schema.clone())
            .build();

        let repo = C3p0JsonRepository::build(conf);

        C3p0Migrate {
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
    C3P0INIT,
    UP,
    DOWN,
}

#[derive(Clone)]
pub struct C3p0Migrate {
    table: String,
    schema: Option<String>,
    migrations: Vec<SqlMigration>,
    repo: C3p0JsonRepository<
        MigrationData,
        DefaultJsonCodec,
        JsonManager<'static, MigrationData, DefaultJsonCodec>,
    >,
}

const C3P0_INIT_MIGRATION_ID: &str = "C3P0_INIT_MIGRATION";

impl C3p0Migrate {
    pub fn migrate(&self, c3p0: &C3p0) -> Result<(), C3p0Error> {
        {
            let conn = c3p0.connection()?;
            if let Err(err) = self.repo.create_table_if_not_exists(&conn) {
                warn!("Create table process completed with error. This 'COULD' be fine if another process attempted the same operation concurrently. Err: {}", err);
            };
        }

        // Start Migration
        c3p0.transaction(|conn| Ok(self.create_migration_zero(conn)?))
            .map_err(C3p0Error::from)?;

        // Start Migration
        c3p0.transaction(|conn| Ok(self.start_migration(conn)?))
            .map_err(C3p0Error::from)
    }

    fn create_migration_zero(&self, conn: &Connection) -> Result<(), C3p0Error> {

        #[cfg(feature = "pg")]
        {
            conn.batch_execute(&format!(
                "LOCK TABLE {} IN ACCESS EXCLUSIVE MODE",
                self.repo.json_manager().qualified_table_name()
            ))?;
        };

        #[cfg(feature = "mysql")]
            {
                conn.batch_execute(&format!(
                    "LOCK TABLES {} WRITE",
                    self.repo.json_manager().qualified_table_name()
                ))?;

            };


        #[cfg(any(feature = "mysql", feature = "sqlite"))]
        let sql = format!(
            r#"select count(*) from {} where JSON_EXTRACT({}, "$.migration_id") = ?"#,
            self.repo.json_manager().qualified_table_name(),
            self.repo.json_manager().data_field_name()
        );

        #[cfg(feature = "pg")]
        let sql = format!(
            r#"select count(*) from {} where {}->>'migration_id' = $1"#,
            self.repo.json_manager().qualified_table_name(),
            self.repo.json_manager().data_field_name()
        );

        let count: i64 = conn.fetch_one_value(&sql, &[&C3P0_INIT_MIGRATION_ID])?;

        if count == 0 {
            let migration_zero = MigrationData {
                md5_checksum: "".to_owned(),
                migration_id: C3P0_INIT_MIGRATION_ID.to_owned(),
                migration_type: MigrationType::C3P0INIT,
                execution_time_ms: 0,
                installed_on_epoch_ms: 0,
                success: true,
            };
            self.repo.save(&conn, migration_zero.into())?;
        };

        Ok(())
    }

    fn start_migration(&self, conn: &Connection) -> Result<(), C3p0Error> {

        #[cfg(feature = "mysql")]
        let lock_sql = format!(
            r#"select * from {} where JSON_EXTRACT({}, "$.migration_id") = ? FOR UPDATE"#,
            self.repo.json_manager().qualified_table_name(),
            self.repo.json_manager().data_field_name()
        );

        #[cfg(feature = "pg")]
        let lock_sql = format!(
            r#"select * from {} where {}->>'migration_id' = $1 FOR UPDATE"#,
            self.repo.json_manager().qualified_table_name(),
            self.repo.json_manager().data_field_name()
        );

        #[cfg(feature = "sqlite")]
        let lock_sql = format!(
            r#"select * from {} where JSON_EXTRACT({}, "$.migration_id") = ?"#,
            self.repo.json_manager().qualified_table_name(),
            self.repo.json_manager().data_field_name()
        );

        conn.fetch_one(&lock_sql, &[&C3P0_INIT_MIGRATION_ID], |_| Ok(()))?;

        let migration_history = self.fetch_migrations_history(conn)?;
        let migration_history = C3p0Migrate::clean_history(migration_history)?;

        for i in 0..self.migrations.len() {
            let migration = &self.migrations[i];

            if migration_history.len() > i {
                let applied_migration = &migration_history[i];

                if applied_migration.data.migration_id.eq(&migration.id) {
                    if applied_migration.data.md5_checksum.eq(&migration.up.md5) {
                        continue;
                    }
                    return Err(C3p0Error::AlteredMigrationSql {
                        message: format!(
                            "Wrong checksum for migration [{}]. Expected [{}], found [{}].",
                            applied_migration.data.migration_id,
                            applied_migration.data.md5_checksum,
                            migration.up.md5
                        ),
                    });
                }
                return Err(C3p0Error::WrongMigrationSet {
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
        conn: &Connection,
    ) -> Result<Vec<MigrationModel>, C3p0Error> {
        self.repo.find_all(conn).map_err(C3p0Error::from)
    }

    fn clean_history(
        migrations: Vec<MigrationModel>,
    ) -> Result<Vec<MigrationModel>, C3p0Error> {
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
}
