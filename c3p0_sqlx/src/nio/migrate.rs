use crate::nio::*;
use c3p0_common::*;

use async_trait::async_trait;

pub trait PgC3p0AsyncMigrateBuilder {
    fn build(self) -> C3p0MigrateAsync<SqlxConnectionAsync, PgC3p0PoolAsync, PgMigratorAsync>;
}

impl PgC3p0AsyncMigrateBuilder for C3p0MigrateBuilder<PgC3p0PoolAsync> {
    fn build(self) -> C3p0MigrateAsync<SqlxConnectionAsync, PgC3p0PoolAsync, PgMigratorAsync> {
        C3p0MigrateAsync::new(
            self.table,
            self.schema,
            self.migrations,
            self.c3p0,
            PgMigratorAsync {},
        )
    }
}

#[derive(Clone)]
pub struct PgMigratorAsync {}

#[async_trait]
impl MigratorAsync for PgMigratorAsync {
    type Conn = SqlxConnectionAsync;
    type C3P0 = PgC3p0PoolAsync;
    type C3P0Json = PgC3p0JsonAsync<MigrationData, DefaultJsonCodec>;

    fn build_cp30_json(
        &self,
        table: String,
        schema: Option<String>,
    ) -> PgC3p0JsonAsync<MigrationData, DefaultJsonCodec> {
        C3p0JsonBuilder::<PgC3p0PoolAsync>::new(table)
            .with_schema_name(schema)
            .build()
    }

    async fn lock_table(
        &self,
        c3p0_json: &PgC3p0JsonAsync<MigrationData, DefaultJsonCodec>,
        conn: &mut SqlxConnectionAsync,
    ) -> Result<(), C3p0Error> {
        conn.batch_execute(&format!(
            "LOCK TABLE {} IN ACCESS EXCLUSIVE MODE",
            c3p0_json.queries().qualified_table_name
        ))
        .await
    }

    async fn lock_first_migration_row(
        &self,
        c3p0_json: &PgC3p0JsonAsync<MigrationData, DefaultJsonCodec>,
        conn: &mut SqlxConnectionAsync,
    ) -> Result<(), C3p0Error> {
        let lock_sql = format!(
            r#"select * from {} where {}->>'migration_id' = $1 FOR UPDATE"#,
            c3p0_json.queries().qualified_table_name,
            c3p0_json.queries().data_field_name
        );
        conn.fetch_one(&lock_sql, &[&C3P0_INIT_MIGRATION_ID], |_| Ok(()))
            .await
    }
}
