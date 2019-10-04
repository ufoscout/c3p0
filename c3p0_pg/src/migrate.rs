use crate::json::{C3p0JsonBuilderPg, C3p0JsonPg};
use crate::pool::{C3p0PoolPg, PgConnection};
use c3p0_common::error::C3p0Error;
use c3p0_common::json::builder::C3p0JsonBuilder;
use c3p0_common::json::codec::DefaultJsonCodec;
use c3p0_common::pool::Connection;

use c3p0_common::migrate::*;

pub trait C3p0MigrateBuilderPg {
    fn build(self) -> C3p0Migrate<C3p0PoolPg, PgMigrator>;
}

impl C3p0MigrateBuilderPg for C3p0MigrateBuilder<C3p0PoolPg> {
    fn build(self) -> C3p0Migrate<C3p0PoolPg, PgMigrator> {
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
    type CONNECTION = PgConnection;
    type C3P0 = C3p0PoolPg;
    type C3P0JSON = C3p0JsonPg<MigrationData, DefaultJsonCodec>;

    fn build_cp30_json(
        &self,
        table: String,
        schema: Option<String>,
    ) -> C3p0JsonPg<MigrationData, DefaultJsonCodec> {
        C3p0JsonBuilder::<C3p0PoolPg>::new(table)
            .with_schema_name(schema)
            .build()
    }

    fn lock_table(
        &self,
        c3p0_json: &C3p0JsonPg<MigrationData, DefaultJsonCodec>,
        conn: &PgConnection,
    ) -> Result<(), C3p0Error> {
        conn.batch_execute(&format!(
            "LOCK TABLE {} IN ACCESS EXCLUSIVE MODE",
            c3p0_json.queries().qualified_table_name
        ))
    }

    fn lock_first_migration_row(
        &self,
        c3p0_json: &C3p0JsonPg<MigrationData, DefaultJsonCodec>,
        conn: &PgConnection,
    ) -> Result<(), C3p0Error> {
        let lock_sql = format!(
            r#"select * from {} where {}->>'migration_id' = $1 FOR UPDATE"#,
            c3p0_json.queries().qualified_table_name,
            c3p0_json.queries().data_field_name
        );
        conn.fetch_one(&lock_sql, &[&C3P0_INIT_MIGRATION_ID], |_| Ok(()))
    }
}
