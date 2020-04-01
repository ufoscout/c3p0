use crate::json::{PgC3p0JsonAsync, PgC3p0JsonAsyncBuilder};
use crate::pool::{PgC3p0PoolAsync, PgConnectionAsync};
use c3p0_common::error::C3p0Error;
use c3p0_common::json::builder::C3p0JsonBuilder;
use c3p0_common::json::codec::DefaultJsonCodec;

use c3p0_common_async::{MigratorAsync, SqlConnectionAsync, C3p0MigrateAsync};

use async_trait::async_trait;
use c3p0_common::{C3p0MigrateBuilder, MigrationData, C3P0_INIT_MIGRATION_ID};

pub trait PgC3p0MigrateBuilder {
    fn build(self) -> C3p0MigrateAsync<PgConnectionAsync, PgC3p0PoolAsync, PgMigratorAsync>;
}

impl PgC3p0MigrateBuilder for C3p0MigrateBuilder<PgC3p0PoolAsync> {
    fn build(self) -> C3p0MigrateAsync<PgConnectionAsync, PgC3p0PoolAsync, PgMigratorAsync> {
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

#[async_trait(?Send)]
impl MigratorAsync for PgMigratorAsync {
    type CONN = PgConnectionAsync;
    type C3P0 = PgC3p0PoolAsync;
    type C3P0JSON = PgC3p0JsonAsync<MigrationData, DefaultJsonCodec>;

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
        conn: &mut PgConnectionAsync,
    ) -> Result<(), C3p0Error> {
        conn.batch_execute(&format!(
            "LOCK TABLE {} IN ACCESS EXCLUSIVE MODE",
            c3p0_json.queries().qualified_table_name
        )).await
    }

    async fn lock_first_migration_row(
        &self,
        c3p0_json: &PgC3p0JsonAsync<MigrationData, DefaultJsonCodec>,
        conn: &mut PgConnectionAsync,
    ) -> Result<(), C3p0Error> {
        let lock_sql = format!(
            r#"select * from {} where {}->>'migration_id' = $1 FOR UPDATE"#,
            c3p0_json.queries().qualified_table_name,
            c3p0_json.queries().data_field_name
        );
        conn.fetch_one(&lock_sql, &[&C3P0_INIT_MIGRATION_ID], |_| Ok(())).await
    }
}
