use c3p0_common::*;

use crate::common::executor::execute;
use crate::{SqlxMySqlC3p0Json, SqlxMySqlC3p0JsonBuilder, SqlxMySqlC3p0Pool, SqlxMySqlConnection};
use async_trait::async_trait;

pub trait SqlxMySqlC3p0MigrateBuilder {
    fn build(self) -> C3p0Migrate<SqlxMySqlConnection, SqlxMySqlC3p0Pool, SqlxMySqlMigrator>;
}

impl SqlxMySqlC3p0MigrateBuilder for C3p0MigrateBuilder<SqlxMySqlC3p0Pool> {
    fn build(self) -> C3p0Migrate<SqlxMySqlConnection, SqlxMySqlC3p0Pool, SqlxMySqlMigrator> {
        C3p0Migrate::new(
            self.table,
            self.schema,
            self.migrations,
            self.c3p0,
            SqlxMySqlMigrator {},
        )
    }
}

#[derive(Clone)]
pub struct SqlxMySqlMigrator {}

#[async_trait]
impl C3p0Migrator for SqlxMySqlMigrator {
    type Conn = SqlxMySqlConnection;
    type C3P0 = SqlxMySqlC3p0Pool;
    type C3P0Json = SqlxMySqlC3p0Json<MigrationData, DefaultJsonCodec>;

    fn build_cp30_json(&self, table: String, schema: Option<String>) -> Self::C3P0Json {
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
            "LOCK TABLES {} WRITE",
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
            r#"select * from {} where JSON_EXTRACT({}, "$.migration_id") = ? FOR UPDATE"#,
            c3p0_json.queries().qualified_table_name,
            c3p0_json.queries().data_field_name
        );
        execute(
            sqlx::query(&lock_sql).bind(C3P0_INIT_MIGRATION_ID),
            conn.get_conn(),
        )
        .await
    }
}
