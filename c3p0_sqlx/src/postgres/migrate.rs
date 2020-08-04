use c3p0_common::*;

use async_trait::async_trait;
use crate::{SqlxPgC3p0Pool, SqlxPgConnection, SqlxPgC3p0Json, SqlxPgC3p0JsonBuilder};
use crate::common::executor::execute;

pub trait SqlxPgC3p0MigrateBuilder {
    fn build(self) -> C3p0Migrate<SqlxPgConnection, SqlxPgC3p0Pool, SqlxPgMigrator>;
}

impl SqlxPgC3p0MigrateBuilder for C3p0MigrateBuilder<SqlxPgC3p0Pool> {
    fn build(self) -> C3p0Migrate<SqlxPgConnection, SqlxPgC3p0Pool, SqlxPgMigrator> {
        C3p0Migrate::new(
            self.table,
            self.schema,
            self.migrations,
            self.c3p0,
            SqlxPgMigrator {},
        )
    }
}

#[derive(Clone)]
pub struct SqlxPgMigrator {}

#[async_trait]
impl C3p0Migrator for SqlxPgMigrator {
    type Conn = SqlxPgConnection;
    type C3P0 = SqlxPgC3p0Pool;
    type C3P0Json = SqlxPgC3p0Json<MigrationData, DefaultJsonCodec>;

    fn build_cp30_json(
        &self,
        table: String,
        schema: Option<String>,
    ) -> Self::C3P0Json {
        C3p0JsonBuilder::<Self::C3P0>::new(table)
            .with_schema_name(schema)
            .build()
    }

    async fn lock_table(
        &self,
        c3p0_json: &Self::C3P0Json,
        conn: &mut Self::Conn,
    ) -> Result<(), C3p0Error> {
        conn.batch_execute(&format!(
            "LOCK TABLE {} IN ACCESS EXCLUSIVE MODE",
            c3p0_json.queries().qualified_table_name
        ))
        .await
    }

    async fn lock_first_migration_row(
        &self,
        c3p0_json: &Self::C3P0Json,
        conn: &mut Self::Conn,
    ) -> Result<(), C3p0Error> {
        let lock_sql = format!(
            r#"select * from {} where {}->>'migration_id' = $1 FOR UPDATE"#,
            c3p0_json.queries().qualified_table_name,
            c3p0_json.queries().data_field_name
        );
        execute(sqlx::query(&lock_sql)
            .bind(C3P0_INIT_MIGRATION_ID), conn.get_conn()).await
    }
}
