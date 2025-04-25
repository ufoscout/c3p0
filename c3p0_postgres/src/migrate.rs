use crate::*;
use c3p0_common::*;

pub trait PgC3p0MigrateBuilder {
    fn build(self) -> C3p0Migrate<PgMigrator>;
}

impl PgC3p0MigrateBuilder for C3p0MigrateBuilder<PgC3p0Pool> {
    fn build(self) -> C3p0Migrate<PgMigrator> {
        C3p0Migrate::new(
            self.migrations,
            self.c3p0,
            PgMigrator {
                c3p0_json: PgC3p0JsonBuilder::<u64, i64>::new(self.table)
                .with_schema_name(self.schema)
                .build(),
            },
        )
    }
}

#[derive(Clone)]
pub struct PgMigrator {
    c3p0_json: PgC3p0Json<u64, i64, MigrationData, DefaultJsonCodec>,
}

impl C3p0Migrator for PgMigrator {
    type Tx<'a> = PgTx<'a>;
    type C3P0 = PgC3p0Pool;
    type C3P0Json = PgC3p0Json<u64, i64, MigrationData, DefaultJsonCodec>;

    fn cp30_json(&self) -> &Self::C3P0Json {
        &self.c3p0_json
    }

    async fn batch_execute(&self, sql: &str, conn: &mut Self::Tx<'_>) -> Result<(), C3p0Error> {
        conn.batch_execute(sql).await
    }

    async fn lock_table(
        &self,
        conn: &mut Self::Tx<'_>,
    ) -> Result<(), C3p0Error> {
        conn.batch_execute(&format!(
            "LOCK TABLE {} IN ACCESS EXCLUSIVE MODE",
            self.c3p0_json.queries().qualified_table_name
        ))
        .await
    }

    async fn lock_first_migration_row(
        &self,
        conn: &mut Self::Tx<'_>,
    ) -> Result<(), C3p0Error> {
        let lock_sql = format!(
            r#"select * from {} where {}->>'migration_id' = $1 FOR UPDATE"#,
            self.c3p0_json.queries().qualified_table_name,
            self.c3p0_json.queries().data_field_name
        );
        conn.fetch_one(&lock_sql, &[&C3P0_INIT_MIGRATION_ID], |_| Ok(()))
            .await
    }
}
