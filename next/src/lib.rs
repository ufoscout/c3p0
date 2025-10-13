#![doc = include_utils::include_md!("README.md")]

pub mod codec;
pub mod error;
pub mod pool;
pub mod record;
pub mod time;

#[cfg(feature = "postgres")]
pub mod postgres;
#[cfg(feature = "mysql")]
pub mod mysql;
#[cfg(feature = "sqlite")]
pub mod sqlite;

pub use sqlx::*;
pub use codec::Codec;
pub use error::C3p0Error;
pub use pool::C3p0Pool;
pub use record::*;

#[cfg(feature = "postgres")]
pub use crate::postgres::PgC3p0Pool;
#[cfg(feature = "mysql")]
pub use crate::mysql::MySqlC3p0Pool;
#[cfg(feature = "sqlite")]
pub use crate::sqlite::SqliteC3p0Pool;