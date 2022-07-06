use c3p0_common::*;
use log::warn;

use crate::{SqlxSqliteC3p0Json, SqlxSqliteC3p0JsonBuilder, SqlxSqliteC3p0Pool, SqlxSqliteConnection};
use async_trait::async_trait;

pub trait SqlxSqliteC3p0MigrateBuilder {
    fn build(self) -> C3p0Migrate<SqlxSqliteConnection, SqlxSqliteC3p0Pool, SqlxSqliteMigrator>;
}

impl SqlxSqliteC3p0MigrateBuilder for C3p0MigrateBuilder<SqlxSqliteC3p0Pool> {
    fn build(self) -> C3p0Migrate<SqlxSqliteConnection, SqlxSqliteC3p0Pool, SqlxSqliteMigrator> {
        C3p0Migrate::new(
            self.table,
            self.schema,
            self.migrations,
            self.c3p0,
            SqlxSqliteMigrator {},
        )
    }
}

#[derive(Clone)]
pub struct SqlxSqliteMigrator {}

#[async_trait]
impl C3p0Migrator for SqlxSqliteMigrator {
    type Conn = SqlxSqliteConnection;
    type C3P0 = SqlxSqliteC3p0Pool;
    type C3P0Json = SqlxSqliteC3p0Json<MigrationData, DefaultJsonCodec>;

    fn build_cp30_json(&self, table: String, schema: Option<String>) -> Self::C3P0Json {
        C3p0JsonBuilder::<Self::C3P0>::new(table)
            .with_schema_name(schema)
            .build()
    }

    async fn lock_table(
        &self,
        _c3p0_json: &Self::C3P0Json,
        _conn: &mut Self::Conn,
    ) -> Result<(), C3p0Error> {
        warn!("SQLite does not support locking table. The table will not be locked.");

        // conn.batch_execute(&format!(
        //     "LOCK TABLE {} IN ACCESS EXCLUSIVE MODE",
        //     c3p0_json.queries().qualified_table_name
        // ))
        // .await
        Ok(())
    }

    async fn lock_first_migration_row(
        &self,
        _c3p0_json: &Self::C3P0Json,
        _conn: &mut Self::Conn,
    ) -> Result<(), C3p0Error> {
        warn!("SQLite does not support 'Select... for Update' statements. The row will not be locked during the migration.");

        // let lock_sql = format!(
        //     r#"select * from {} where {}->>'migration_id' = $1 FOR UPDATE"#,
        //     c3p0_json.queries().qualified_table_name,
        //     c3p0_json.queries().data_field_name
        // );
        // execute(
        //     sqlx::query(&lock_sql).bind(C3P0_INIT_MIGRATION_ID),
        //     conn.get_conn(),
        // )
        // .await
        Ok(())
    }
}
