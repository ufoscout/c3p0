mod json;
mod pool;
mod queries;

#[cfg(feature = "migrate")]
mod migrate;
#[cfg(feature = "migrate")]
pub use migrate::*;
use crate::common::executor::execute;

pub use json::*;
pub use pool::*;

type Db = sqlx::sqlite::Sqlite;
type DbRow = sqlx::sqlite::SqliteRow;
