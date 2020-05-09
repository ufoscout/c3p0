use crate::nio::*;
use c3p0_common::*;

use async_trait::async_trait;

pub trait MysqlC3p0AsyncMigrateBuilder {
    fn build(self) -> C3p0MigrateAsync<MysqlConnectionAsync, MysqlC3p0PoolAsync, MysqlMigratorAsync>;
}

impl MysqlC3p0AsyncMigrateBuilder for C3p0MigrateBuilder<MysqlC3p0PoolAsync> {
    fn build(self) -> C3p0MigrateAsync<MysqlConnectionAsync, MysqlC3p0PoolAsync, MysqlMigratorAsync> {
        C3p0MigrateAsync::new(
            self.table,
            self.schema,
            self.migrations,
            self.c3p0,
            MysqlMigratorAsync {},
        )
    }
}

#[derive(Clone)]
pub struct MysqlMigratorAsync {}

#[async_trait]
impl MigratorAsync for MysqlMigratorAsync {
    type Conn = MysqlConnectionAsync;
    type C3P0 = MysqlC3p0PoolAsync;
    type C3P0Json = MysqlC3p0JsonAsync<MigrationData, DefaultJsonCodec>;

    fn build_cp30_json(
        &self,
        table: String,
        schema: Option<String>,
    ) -> MysqlC3p0JsonAsync<MigrationData, DefaultJsonCodec> {
        C3p0JsonBuilder::<MysqlC3p0PoolAsync>::new(table)
            .with_schema_name(schema)
            .build()
    }

    async fn lock_table(
        &self,
        c3p0_json: &MysqlC3p0JsonAsync<MigrationData, DefaultJsonCodec>,
        conn: &mut MysqlConnectionAsync,
    ) -> Result<(), C3p0Error> {
        conn.batch_execute(&format!(
            "LOCK TABLES {} WRITE",
            c3p0_json.queries().qualified_table_name
        ))
        .await
    }

    async fn lock_first_migration_row(
        &self,
        c3p0_json: &MysqlC3p0JsonAsync<MigrationData, DefaultJsonCodec>,
        conn: &mut MysqlConnectionAsync,
    ) -> Result<(), C3p0Error> {
        let lock_sql = format!(
            r#"select * from {} where JSON_EXTRACT({}, "$.migration_id") = ? FOR UPDATE"#,
            c3p0_json.queries().qualified_table_name,
            c3p0_json.queries().data_field_name
        );
        conn.fetch_one(&lock_sql, &[&C3P0_INIT_MIGRATION_ID], |_| Ok(()))
            .await
    }
}
