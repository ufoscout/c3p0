mod json;
mod pool;
mod queries;

pub use json::*;
pub use pool::*;

type Db = sqlx::mysql::MySql;
type DbRow = sqlx::mysql::MySqlRow;
