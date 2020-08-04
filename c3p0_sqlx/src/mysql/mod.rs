mod json;
mod pool;
mod queries;

#[cfg(feature = "migrate")]
mod migrate;
#[cfg(feature = "migrate")]
pub use migrate::*;

pub use json::*;
pub use pool::*;

type Db = sqlx::mysql::MySql;
type DbRow = sqlx::mysql::MySqlRow;
