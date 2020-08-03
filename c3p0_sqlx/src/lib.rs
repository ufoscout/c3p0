mod common;
mod error;
mod json;
mod pool;

pub use common::*;
pub use error::*;
pub use json::*;
pub use pool::*;

pub mod sqlx {
    pub use sqlx::*;
}

/*
#[cfg(feature = "migrate")]
mod migrate;
#[cfg(feature = "migrate")]
pub use migrate::*;
*/

#[cfg(any(feature = "postgres"))]
type Db = sqlx::postgres::Postgres;
#[cfg(any(feature = "postgres"))]
type DbRow = sqlx::postgres::PgRow;
