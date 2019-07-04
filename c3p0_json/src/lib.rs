pub mod client;
pub mod json;

#[cfg(feature = "mysql")]
pub use crate::client::mysql::{C3p0MysqlJson, C3p0MysqlJsonBuilder};
#[cfg(feature = "mysql")]
pub mod mysql {
    pub use c3p0_mysql::*;
}

#[cfg(feature = "pg")]
pub use crate::client::pg::{C3p0PgJson, C3p0PgJsonBuilder};
#[cfg(feature = "pg")]
pub mod pg {
    pub use c3p0_pg::*;
}

#[cfg(feature = "sqlite")]
pub use crate::client::sqlite::{C3p0SqliteJson, C3p0SqliteJsonBuilder};
#[cfg(feature = "sqlite")]
pub mod sqlite {
    pub use c3p0_sqlite::*;
}

pub use crate::json::{codec::JsonCodec, model::Model, model::NewModel, C3p0Json};
pub use c3p0_common::error::C3p0Error;
pub use c3p0_common::pool::Connection;
