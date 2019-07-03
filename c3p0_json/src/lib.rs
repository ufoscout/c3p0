pub mod client;
pub mod json;

#[cfg(feature = "mysql")]
pub use crate::client::mysql::{C3p0MysqlJson, C3p0MysqlJsonBuilder};
#[cfg(feature = "mysql")]
pub use c3p0_mysql::{C3p0Mysql, C3p0MysqlBuilder, MySqlConnection};

#[cfg(feature = "pg")]
pub use crate::client::pg::{C3p0PgJson, C3p0PgJsonBuilder};
#[cfg(feature = "pg")]
pub use c3p0_pg::{C3p0Pg, C3p0PgBuilder, PgConnection};

#[cfg(feature = "sqlite")]
pub use crate::client::sqlite::{C3p0SqliteJson, C3p0SqliteJsonBuilder};
#[cfg(feature = "sqlite")]
pub use c3p0_sqlite::{C3p0Sqlite, C3p0SqliteBuilder, SqliteConnection};

pub use crate::json::{codec::JsonCodec, model::Model, model::NewModel, C3p0Json};
pub use c3p0_common::error::C3p0Error;
