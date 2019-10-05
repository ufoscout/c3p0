use crate::json::{C3p0JsonBuilderMysql, C3p0JsonMysql};
use crate::mysql::{C3p0PoolMysql, MysqlConnection};
use c3p0_common::error::C3p0Error;
use c3p0_common::json::builder::C3p0JsonBuilder;
use c3p0_common::json::codec::DefaultJsonCodec;
use c3p0_common::pool::SqlConnection;

pub use c3p0_common::migrate::*;

pub trait C3p0MigrateBuilderMysql {
    fn build(self) -> C3p0Migrate<MysqlConnection, C3p0PoolMysql, MysqlMigrator>;
}

impl C3p0MigrateBuilderMysql for C3p0MigrateBuilder<MysqlConnection, C3p0PoolMysql> {
    fn build(self) -> C3p0Migrate<MysqlConnection, C3p0PoolMysql, MysqlMigrator> {
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
    type C3P0 = C3p0PoolMysql;
    type C3P0JSON = C3p0JsonMysql<MigrationData, DefaultJsonCodec>;

    fn build_cp30_json(
        &self,
        table: String,
        schema: Option<String>,
    ) -> C3p0JsonMysql<MigrationData, DefaultJsonCodec> {
        C3p0JsonBuilder::<C3p0PoolMysql>::new(table)
            .with_schema_name(schema)
            .build()
    }

    fn lock_table(
        &self,
        c3p0_json: &C3p0JsonMysql<MigrationData, DefaultJsonCodec>,
        conn: &MysqlConnection,
    ) -> Result<(), C3p0Error> {
        conn.batch_execute(&format!(
            "LOCK TABLES {} WRITE",
            c3p0_json.queries().qualified_table_name
        ))
    }

    fn lock_first_migration_row(
        &self,
        c3p0_json: &C3p0JsonMysql<MigrationData, DefaultJsonCodec>,
        conn: &MysqlConnection,
    ) -> Result<(), C3p0Error> {
        let lock_sql = format!(
            r#"select * from {} where JSON_EXTRACT({}, "$.migration_id") = ? FOR UPDATE"#,
            c3p0_json.queries().qualified_table_name,
            c3p0_json.queries().data_field_name
        );
        conn.fetch_one(&lock_sql, &[&C3P0_INIT_MIGRATION_ID], |_| Ok(()))
    }
}
