mod pool;

pub use pool::*;

type Db = sqlx::mysql::MySql;
type DbRow = sqlx::mysql::MySqlRow;
