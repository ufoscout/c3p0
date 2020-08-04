mod queries;
mod json;
mod pool;

pub use json::*;
pub use pool::*;

type Db = sqlx::mysql::MySql;
type DbRow = sqlx::mysql::MySqlRow;
