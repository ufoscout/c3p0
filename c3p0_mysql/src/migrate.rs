use crate::json::{MysqlC3p0Json, MysqlC3p0JsonBuilder};
use crate::mysql::{MysqlC3p0Pool, MysqlConnection};
use c3p0_common::error::C3p0Error;
use c3p0_common::json::builder::C3p0JsonBuilder;
use c3p0_common::json::codec::DefaultJsonCodec;
use c3p0_common::pool::SqlConnection;

use c3p0_common::migrate::*;

pub trait MysqlC3p0MigrateBuilder {
    fn build(self) -> C3p0Migrate<MysqlConnection, MysqlC3p0Pool, MysqlMigrator>;
}

impl MysqlC3p0MigrateBuilder for C3p0MigrateBuilder<MysqlC3p0Pool> {
    fn build(self) -> C3p0Migrate<MysqlConnection, MysqlC3p0Pool, MysqlMigrator> {
        C3p0Migrate::new(
            self.table,
            self.schema,
            self.migrations,
            self.c3p0,
            MysqlMigrator {},
        )
    }
}

#[derive(Clone)]
pub struct MysqlMigrator {}

impl Migrator for MysqlMigrator {
    type CONN = MysqlConnection;
    type C3P0 = MysqlC3p0Pool;
    type C3P0JSON = MysqlC3p0Json<MigrationData, DefaultJsonCodec>;

    fn build_cp30_json(
        &self,
        table: String,
        schema: Option<String>,
    ) -> MysqlC3p0Json<MigrationData, DefaultJsonCodec> {
        C3p0JsonBuilder::<MysqlC3p0Pool>::new(table)
            .with_schema_name(schema)
            .build()
    }

    fn lock_table(
        &self,
        c3p0_json: &MysqlC3p0Json<MigrationData, DefaultJsonCodec>,
        conn: &mut MysqlConnection,
    ) -> Result<(), C3p0Error> {
        conn.batch_execute(&format!(
            "LOCK TABLES {} WRITE",
            c3p0_json.queries().qualified_table_name
        ))
    }

    fn lock_first_migration_row(
        &self,
        c3p0_json: &MysqlC3p0Json<MigrationData, DefaultJsonCodec>,
        conn: &mut MysqlConnection,
    ) -> Result<(), C3p0Error> {
        let lock_sql = format!(
            r#"select * from {} where JSON_EXTRACT({}, "$.migration_id") = ? FOR UPDATE"#,
            c3p0_json.queries().qualified_table_name,
            c3p0_json.queries().data_field_name
        );
        conn.fetch_one(&lock_sql, &[&C3P0_INIT_MIGRATION_ID], |_| Ok(()))
    }
}
