use crate::migration::{Migration, Migrations};
use crate::sql_migration::{to_sql_migrations, SqlMigration};
use log::*;
use serde_derive::{Deserialize, Serialize};
use c3p0_common::pool::{Connection, C3p0PoolManager};
use c3p0_common::json::model::{Model, NewModel};
use c3p0_common::error::C3p0Error;
use c3p0_common::json::codec::DefaultJsonCodec;
use c3p0_common::json::{C3p0JsonManager, C3p0Json};
use c3p0_common::C3p0Pool;
use c3p0_common::json::builder::C3p0JsonBuilder;

mod md5;
pub mod migration;
mod sql_migration;

pub const C3P0_MIGRATE_TABLE_DEFAULT: &str = "C3P0_MIGRATE_SCHEMA_HISTORY";

#[derive(Clone, Debug)]
pub struct C3p0MigrateBuilder<C3P0: C3p0PoolManager> {
    table: String,
    schema: Option<String>,
    migrations: Vec<Migration>,
    c3p0: C3p0Pool<C3P0>,
}

impl<C3P0: C3p0PoolManager> C3p0MigrateBuilder<C3P0> {
    pub fn new(c3p0: C3p0Pool<C3P0>) -> Self {
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
        self.migrations = migrations.into().migrations;
        self
    }

    pub fn build(self) -> C3p0Migrate<C3P0> {
        C3p0Migrate {
            table: self.table,
            schema: self.schema,
            migrations: to_sql_migrations(self.migrations),
            c3p0: self.c3p0,
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
pub struct C3p0Migrate<C3P0: C3p0PoolManager> {
    table: String,
    schema: Option<String>,
    migrations: Vec<SqlMigration>,
    c3p0: C3p0Pool<C3P0>,
}

const C3P0_INIT_MIGRATION_ID: &str = "C3P0_INIT_MIGRATION";

#[cfg(feature = "pg")]
impl C3p0Migrate<c3p0_pool_pg::PgPoolManager> {
    pub fn migrate(&self) -> Result<(), C3p0Error> {
        let c3p0_json = self.build_cp30_json();

        {
            let conn = self.c3p0.connection()?;
            if let Err(err) = c3p0_json.create_table_if_not_exists(&conn) {
                warn!("Create table process completed with error. This 'COULD' be fine if another process attempted the same operation concurrently. Err: {}", err);
            };
        }

        // Start Migration
        self.c3p0.transaction(|conn| {
            self.lock_table(&c3p0_json, conn)?;
            Ok(self.create_migration_zero(&c3p0_json, conn)?)
        })?;

        // Start Migration
        self.c3p0.transaction(|conn| {
            self.lock_first_migration_row(&c3p0_json, conn)?;
            Ok(self.start_migration(&c3p0_json, conn)?)
        })
    }

    pub fn get_migrations_history(
        &self,
        conn: &c3p0_pool_pg::PgConnection,
    ) -> Result<Vec<MigrationModel>, C3p0Error> {
        let c3p0_json = self.build_cp30_json();
        c3p0_json.find_all(conn)
    }

    fn lock_table(
        &self,
        c3p0_json: &C3p0Json<MigrationData, DefaultJsonCodec, c3p0_pool_pg::json::PgJsonManager<MigrationData, DefaultJsonCodec>>,
        conn: &c3p0_pool_pg::PgConnection,
    ) -> Result<(), C3p0Error> {
        conn.batch_execute(&format!(
            "LOCK TABLE {} IN ACCESS EXCLUSIVE MODE",
            c3p0_json.queries().qualified_table_name
        ))
    }

    fn lock_first_migration_row(
        &self,
        c3p0_json: &C3p0Json<MigrationData, DefaultJsonCodec, c3p0_pool_pg::json::PgJsonManager<MigrationData, DefaultJsonCodec>>,
        conn: &c3p0_pool_pg::PgConnection,
    ) -> Result<(), C3p0Error> {
        let lock_sql = format!(
            r#"select * from {} where {}->>'migration_id' = $1 FOR UPDATE"#,
            c3p0_json.queries().qualified_table_name,
            c3p0_json.queries().data_field_name
        );
        conn.fetch_one(&lock_sql, &[&C3P0_INIT_MIGRATION_ID], |_| Ok(()))
    }

    fn build_cp30_json(&self) -> C3p0Json<MigrationData, DefaultJsonCodec, c3p0_pool_pg::json::PgJsonManager<MigrationData, DefaultJsonCodec>> {

        use c3p0_pool_pg::json::PgJsonBuilder;

        C3p0JsonBuilder::new(self.table.clone())
            .with_schema_name(self.schema.clone())
            .build()
    }
}

#[cfg(feature = "mysql")]
impl C3p0Migrate<c3p0_pool_mysql::MysqlPoolManager> {
    pub fn migrate(&self) -> Result<(), C3p0Error> {
        let c3p0_json = self.build_cp30_json();

        {
            let conn = self.c3p0.connection()?;
            if let Err(err) = c3p0_json.create_table_if_not_exists(&conn) {
                warn!("Create table process completed with error. This 'COULD' be fine if another process attempted the same operation concurrently. Err: {}", err);
            };
        }

        // Start Migration
        self.c3p0.transaction(|conn| {
            self.lock_table(&c3p0_json, conn)?;
            Ok(self.create_migration_zero(&c3p0_json, conn)?)
        })?;

        // Start Migration
        self.c3p0.transaction(|conn| {
            self.lock_first_migration_row(&c3p0_json, conn)?;
            Ok(self.start_migration(&c3p0_json, conn)?)
        })
    }

    pub fn get_migrations_history(
        &self,
        conn: &c3p0_pool_mysql::MysqlConnection,
    ) -> Result<Vec<MigrationModel>, C3p0Error> {
        let c3p0_json = self.build_cp30_json();
        c3p0_json.find_all(conn)
    }

    fn lock_table(
        &self,
        c3p0_json: &C3p0Json<MigrationData, DefaultJsonCodec, c3p0_pool_mysql::json::MysqlJsonManager<MigrationData, DefaultJsonCodec>>,
        conn: &c3p0_pool_mysql::MysqlConnection,
    ) -> Result<(), C3p0Error> {
        conn.batch_execute(&format!(
            "LOCK TABLES {} WRITE",
            c3p0_json.queries().qualified_table_name
        ))
    }

    fn lock_first_migration_row(
        &self,
        c3p0_json: &C3p0Json<MigrationData, DefaultJsonCodec, c3p0_pool_mysql::json::MysqlJsonManager<MigrationData, DefaultJsonCodec>>,
        conn: &c3p0_pool_mysql::MysqlConnection,
    ) -> Result<(), C3p0Error> {
        let lock_sql = format!(
            r#"select * from {} where JSON_EXTRACT({}, "$.migration_id") = ? FOR UPDATE"#,
            c3p0_json.queries().qualified_table_name,
            c3p0_json.queries().data_field_name
        );
        conn.fetch_one(&lock_sql, &[&C3P0_INIT_MIGRATION_ID], |_| Ok(()))
    }

    fn build_cp30_json(&self) -> C3p0Json<MigrationData, DefaultJsonCodec, c3p0_pool_mysql::json::MysqlJsonManager<MigrationData, DefaultJsonCodec>> {
        use c3p0_pool_mysql::json::MysqlJsonBuilder;
        C3p0JsonBuilder::new(self.table.clone())
            .with_schema_name(self.schema.clone())
            .build()
    }
}

#[cfg(feature = "sqlite")]
impl C3p0Migrate<c3p0_pool_sqlite::SqlitePoolManager> {
    pub fn migrate(&self) -> Result<(), C3p0Error> {
        let c3p0_json = self.build_cp30_json();

        {
            let conn = self.c3p0.connection()?;
            if let Err(err) = c3p0_json.create_table_if_not_exists(&conn) {
                warn!("Create table process completed with error. This 'COULD' be fine if another process attempted the same operation concurrently. Err: {}", err);
            };
        }

        // Start Migration
        self.c3p0
            .transaction(|conn| Ok(self.create_migration_zero(&c3p0_json, conn)?))?;

        // Start Migration
        self.c3p0.transaction(|conn| {
            self.lock_first_migration_row(&c3p0_json, conn)?;
            Ok(self.start_migration(&c3p0_json, conn)?)
        })
    }

    pub fn get_migrations_history(
        &self,
        conn: &c3p0_pool_sqlite::SqliteConnection,
    ) -> Result<Vec<MigrationModel>, C3p0Error> {
        let c3p0_json = self.build_cp30_json();
        c3p0_json.find_all(conn)
    }

    fn lock_first_migration_row(
        &self,
        c3p0_json: &C3p0Json<MigrationData, DefaultJsonCodec, c3p0_pool_sqlite::json::SqliteJsonManager<MigrationData, DefaultJsonCodec>>,
        conn: &c3p0_pool_sqlite::SqliteConnection,
    ) -> Result<(), C3p0Error> {
        let lock_sql = format!(
            r#"select * from {} where JSON_EXTRACT({}, "$.migration_id") = ?"#,
            c3p0_json.queries().qualified_table_name,
            c3p0_json.queries().data_field_name
        );
        conn.fetch_one(&lock_sql, &[&C3P0_INIT_MIGRATION_ID], |_| Ok(()))
    }

    fn build_cp30_json(&self) -> C3p0Json<MigrationData, DefaultJsonCodec, c3p0_pool_sqlite::json::SqliteJsonManager<MigrationData, DefaultJsonCodec>> {
        use c3p0_pool_sqlite::json::SqliteJsonBuilder;
        C3p0JsonBuilder::new(self.table.clone())
            .with_schema_name(self.schema.clone())
            .build()
    }
}

impl<C3P0: C3p0PoolManager> C3p0Migrate<C3P0> {
    fn create_migration_zero<C3P0JSON: C3p0JsonManager<MigrationData, DefaultJsonCodec>>(
        &self,
        c3p0_json: &C3p0Json<MigrationData, DefaultJsonCodec, C3P0JSON>,
        conn: &C3P0JSON::CONNECTION,
    ) -> Result<(), C3p0Error> {
        let count = c3p0_json.count_all(&conn)?;

        if count == 0 {
            let migration_zero = MigrationData {
                md5_checksum: "".to_owned(),
                migration_id: C3P0_INIT_MIGRATION_ID.to_owned(),
                migration_type: MigrationType::C3P0INIT,
                execution_time_ms: 0,
                installed_on_epoch_ms: 0,
                success: true,
            };
            c3p0_json.save(&conn, migration_zero.into())?;
        };

        Ok(())
    }

    fn start_migration<C3P0JSON: C3p0JsonManager<MigrationData, DefaultJsonCodec>>(
        &self,
        c3p0_json: &C3p0Json<MigrationData, DefaultJsonCodec, C3P0JSON>,
        conn: &C3P0JSON::CONNECTION,
    ) -> Result<(), C3p0Error> {
        let migration_history = self.fetch_migrations_history(c3p0_json, conn)?;
        let migration_history = self.clean_history(migration_history)?;

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

    fn fetch_migrations_history<C3P0JSON: C3p0JsonManager<MigrationData, DefaultJsonCodec>>(
        &self,
        c3p0_json: &C3p0Json<MigrationData, DefaultJsonCodec, C3P0JSON>,
        conn: &C3P0JSON::CONNECTION,
    ) -> Result<Vec<MigrationModel>, C3p0Error> {
        c3p0_json.find_all(conn)
    }

    fn clean_history(
        &self,
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
