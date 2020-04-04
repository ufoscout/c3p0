pub mod error;
pub mod json;
pub mod sql;
pub mod types;

#[cfg(feature = "migrate")]
mod migrate;

mod common {
    pub use crate::error::C3p0Error;
    pub use crate::json::{
        builder::C3p0JsonBuilder, codec::DefaultJsonCodec, codec::JsonCodec, model::Model,
        model::NewModel, model::IdType, model::VersionType
    };
    pub use crate::sql::{ForUpdate, OrderBy};

    #[cfg(feature = "migrate")]
    pub use crate::migrate::{
        C3P0_INIT_MIGRATION_ID, C3p0MigrateBuilder, C3P0_MIGRATE_TABLE_DEFAULT, from_embed, from_fs, Migrations, Migration,
        MigrationData, MigrationModel, MigrationType, include_dir
    };

}

pub use crate::common::*;

#[cfg(feature = "async")]
mod nio;
#[cfg(feature = "async")]
pub use nio::*;

#[cfg(feature = "blocking")]
pub mod blocking;

