use crate::json::{C3p0JsonBuilderSqlite, C3p0JsonSqlite};
use crate::sqlite::{C3p0PoolSqlite, SqliteConnection};
use c3p0_common::error::C3p0Error;
use c3p0_common::json::builder::C3p0JsonBuilder;
use c3p0_common::json::codec::DefaultJsonCodec;

pub use c3p0_common::migrate::*;

pub trait C3p0MigrateBuilderSqlite {
    fn build(self) -> C3p0Migrate<C3p0PoolSqlite, SqliteMigrator>;
}

impl C3p0MigrateBuilderSqlite for C3p0MigrateBuilder<C3p0PoolSqlite> {
    fn build(self) -> C3p0Migrate<C3p0PoolSqlite, SqliteMigrator> {
        C3p0Migrate::new(
            self.table,
            self.schema,
            self.migrations,
            self.c3p0,
            SqliteMigrator {},
        )
    }
}

#[derive(Clone)]
pub struct SqliteMigrator {}

impl Migrator for SqliteMigrator {
    type CONNECTION = SqliteConnection;
    type C3P0 = C3p0PoolSqlite;
    type C3P0JSON = C3p0JsonSqlite<MigrationData, DefaultJsonCodec>;

    fn build_cp30_json(
        &self,
        table: String,
        schema: Option<String>,
    ) -> C3p0JsonSqlite<MigrationData, DefaultJsonCodec> {
        C3p0JsonBuilder::<C3p0PoolSqlite>::new(table)
            .with_schema_name(schema)
            .build()
    }

    fn lock_table(
        &self,
        _c3p0_json: &C3p0JsonSqlite<MigrationData, DefaultJsonCodec>,
        _conn: &SqliteConnection,
    ) -> Result<(), C3p0Error> {
        Ok(())
    }

    fn lock_first_migration_row(
        &self,
        c3p0_json: &C3p0JsonSqlite<MigrationData, DefaultJsonCodec>,
        conn: &SqliteConnection,
    ) -> Result<(), C3p0Error> {
        let lock_sql = format!(
            r#"select * from {} where JSON_EXTRACT({}, "$.migration_id") = ?"#,
            c3p0_json.queries().qualified_table_name,
            c3p0_json.queries().data_field_name
        );
        conn.fetch_one(&lock_sql, &[&C3P0_INIT_MIGRATION_ID], |_| Ok(()))
    }
}
