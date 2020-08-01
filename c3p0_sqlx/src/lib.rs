pub mod common;

mod nio;
pub use nio::*;

#[cfg(any(feature = "postgres"))]
type Db = sqlx::postgres::Postgres;
#[cfg(any(feature = "postgres"))]
type DbRow = sqlx::postgres::PgRow;