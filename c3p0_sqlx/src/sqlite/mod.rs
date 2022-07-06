mod json;
mod pool;
mod queries;

#[cfg(feature = "migrate")]
mod migrate;
use crate::common::executor::execute;
#[cfg(feature = "migrate")]
pub use migrate::*;

pub use json::*;
pub use pool::*;

type Db = sqlx::sqlite::Sqlite;
type DbRow = sqlx::sqlite::SqliteRow;
