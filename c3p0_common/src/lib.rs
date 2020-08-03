pub mod error;
pub mod json;
pub mod pool;
pub mod sql;
pub mod types;

#[cfg(feature = "migrate")]
mod migrate;

mod common {
    pub use crate::error::C3p0Error;
    pub use crate::json::{
        builder::C3p0JsonBuilder, codec::DefaultJsonCodec, codec::JsonCodec, model::IdType,
        model::Model, model::NewModel, model::VersionType, C3p0Json,
    };
    pub use crate::sql::{ForUpdate, OrderBy};

    pub use crate::pool::{C3p0Pool, SqlConnection};

    #[cfg(feature = "migrate")]
    pub use crate::migrate::{
        from_embed, from_fs, include_dir, C3p0Migrate, C3p0MigrateBuilder, C3p0Migrator, Migration,
        MigrationData, MigrationModel, MigrationType, Migrations, C3P0_INIT_MIGRATION_ID,
        C3P0_MIGRATE_TABLE_DEFAULT,
    };
}

pub use crate::common::*;
