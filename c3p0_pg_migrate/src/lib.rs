use crate::migration::Migration;
use postgres::rows::Row;
use postgres::Connection;

mod md5;
pub mod migration;

const C3P0_MIGRATE_TABLE_DEFAULT: &str = "C3P0_MIGRATE_SCHEMA_HISTORY";
const C3P0_MIGRATE_SCHEMA_DEFAULT: &str = "public";

#[derive(Clone, Debug, PartialEq)]
pub struct PgMigrate {
    table: String,
    schema: String,
    migrations: Vec<Migration>,
}

impl PgMigrate {
    pub fn new() -> PgMigrate {
        PgMigrate {
            table: C3P0_MIGRATE_TABLE_DEFAULT.to_owned(),
            schema: C3P0_MIGRATE_SCHEMA_DEFAULT.to_owned(),
            migrations: vec![],
        }
    }
}

/*
Create Table from Flyway:

CREATE TABLE public."MARKET_FLYWAY_SCHEMA_HISTORY" (
    installed_rank int4 NOT NULL,
    "version" varchar(50) NULL,
    description varchar(200) NOT NULL,
    "type" varchar(20) NOT NULL,
    script varchar(1000) NOT NULL,
    checksum int4 NULL,
    installed_by varchar(100) NOT NULL,
    installed_on timestamp NOT NULL DEFAULT now(),
    execution_time int4 NOT NULL,
    success bool NOT NULL,
    CONSTRAINT "MARKET_FLYWAY_SCHEMA_HISTORY_pk" PRIMARY KEY (installed_rank)
)
WITH (
    OIDS=FALSE
) ;
CREATE INDEX "MARKET_FLYWAY_SCHEMA_HISTORY_s_idx" ON public."MARKET_FLYWAY_SCHEMA_HISTORY" USING btree (success) ;

*/

struct MigrationModel {
    installed_order: u32,
    migration_id: String,
    migration_type: MigrationType,
    md5_checksum: String,
    installed_on_epoch_ms: u64,
    execution_time_ms: u64,
    success: bool,
}

enum MigrationType {
    UP,
    DOWN,
}

fn create_migration_table_sql(schema_name: &str, table_name: &str) -> String {
    format!(
        r#"
    CREATE TABLE IF NOT EXISTS {}."{}" (
        installed_order int4 NOT NULL,
        migration_id varchar(1024) NOT NULL,
        migration_type varchar(100) NOT NULL,
        md5_checksum varchar(1024) NOT NULL,
        installed_on_epoch_ms int8 NOT NULL,
        execution_time_ms int8 NOT NULL,
        success bool NOT NULL,
        CONSTRAINT "{}_pk" PRIMARY KEY (installed_order)
    )
    "#,
        schema_name, table_name, table_name
    )
}
