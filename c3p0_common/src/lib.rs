pub mod error;
pub mod json;
pub mod pool;
pub mod sql;
pub mod time;

#[cfg(feature = "migrate")]
mod migrate;

mod common {
    pub use crate::error::C3p0Error;
    pub use crate::json::{
        C3p0Json, codec::DefaultJsonCodec, codec::JsonCodec, model::Model, model::NewModel,
        types::*,
    };
    pub use crate::sql::OrderBy;

    pub use crate::pool::C3p0Pool;

    #[cfg(feature = "migrate")]
    pub use crate::migrate::{
        C3P0_INIT_MIGRATION_ID, C3P0_MIGRATE_TABLE_DEFAULT, C3p0Migrate, C3p0MigrateBuilder,
        C3p0Migrator, Migration, MigrationData, MigrationModel, MigrationType, Migrations,
        from_embed, from_fs, include_dir, sql_migration::SqlMigration, sql_migration::to_sql_migrations, build_migration_zero, clean_history, check_if_migration_already_applied
    };
}

pub use crate::common::*;
