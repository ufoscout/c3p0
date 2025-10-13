mod pool;
mod record;

pub use pool::*;

type Db = sqlx::postgres::Postgres;
type DbRow = sqlx::postgres::PgRow;
