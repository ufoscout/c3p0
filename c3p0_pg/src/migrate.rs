use crate::json::{PgC3p0Json, PgC3p0JsonBuilder};
use crate::pool::{PgC3p0Pool, PgConnection};
use c3p0_common::error::C3p0Error;
use c3p0_common::json::builder::C3p0JsonBuilder;
use c3p0_common::json::codec::DefaultJsonCodec;
use c3p0_common::pool::SqlConnection;

use c3p0_common::migrate::*;

pub trait PgC3p0MigrateBuilder {
    fn build(self) -> C3p0Migrate<PgConnection, PgC3p0Pool, PgMigrator>;
}

impl PgC3p0MigrateBuilder for C3p0MigrateBuilder<PgC3p0Pool> {
    fn build(self) -> C3p0Migrate<PgConnection, PgC3p0Pool, PgMigrator> {
        C3p0Migrate::new(
            self.table,
            self.schema,
            self.migrations,
            self.c3p0,
            PgMigrator {},
        )
    }
}

#[derive(Clone)]
pub struct PgMigrator {}

impl Migrator for PgMigrator {
    type CONN = PgConnection;
    type C3P0 = PgC3p0Pool;
    type C3P0JSON = PgC3p0Json<MigrationData, DefaultJsonCodec>;

    fn build_cp30_json(
        &self,
        table: String,
        schema: Option<String>,
    ) -> PgC3p0Json<MigrationData, DefaultJsonCodec> {
        C3p0JsonBuilder::<PgC3p0Pool>::new(table)
            .with_schema_name(schema)
            .build()
    }

    fn lock_table(
        &self,
        c3p0_json: &PgC3p0Json<MigrationData, DefaultJsonCodec>,
        conn: &mut PgConnection,
    ) -> Result<(), C3p0Error> {
        conn.batch_execute(&format!(
            "LOCK TABLE {} IN ACCESS EXCLUSIVE MODE",
            c3p0_json.queries().qualified_table_name
        ))
    }

    fn lock_first_migration_row(
        &self,
        c3p0_json: &PgC3p0Json<MigrationData, DefaultJsonCodec>,
        conn: &mut PgConnection,
    ) -> Result<(), C3p0Error> {
        let lock_sql = format!(
            r#"select * from {} where {}->>'migration_id' = $1 FOR UPDATE"#,
            c3p0_json.queries().qualified_table_name,
            c3p0_json.queries().data_field_name
        );
        conn.fetch_one(&lock_sql, &[&C3P0_INIT_MIGRATION_ID], |_| Ok(()))
    }
}
