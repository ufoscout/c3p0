pub mod error;
pub mod json;
pub mod sql;
pub mod types;
pub mod pool;

#[cfg(feature = "migrate")]
mod migrate;

mod common {
    pub use crate::error::C3p0Error;
    pub use crate::json::{
        C3p0Json,
        builder::C3p0JsonBuilder, codec::DefaultJsonCodec, codec::JsonCodec, model::IdType,
        model::Model, model::NewModel, model::VersionType,
    };
    pub use crate::sql::{ForUpdate, OrderBy};

    pub use crate::pool::{SqlConnection, C3p0Pool};

    #[cfg(feature = "migrate")]
    pub use crate::migrate::{
        from_embed, from_fs, include_dir, C3p0MigrateBuilder, Migration, MigrationData,
        MigrationModel, MigrationType, Migrations, C3P0_INIT_MIGRATION_ID,
        C3P0_MIGRATE_TABLE_DEFAULT, C3p0Migrate, C3p0Migrator
    };
}

pub use crate::common::*;

