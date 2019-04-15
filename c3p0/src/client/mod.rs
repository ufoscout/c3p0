#[cfg(feature = "mysql")]
mod mysql;

#[cfg(feature = "mysql")]
pub type JsonManager<'a, DATA> = mysql::json::MySqlJsonManager<'a, DATA>;
#[cfg(feature = "mysql")]
pub type JsonManagerBuilder<DATA> = mysql::json::MySqlJsonManagerBuilder<DATA>;
#[cfg(feature = "mysql")]
pub type ToSql = mysql_client::Value;

#[cfg(feature = "pg")]
mod pg;

#[cfg(feature = "pg")]
pub type JsonManager<'a, DATA> = pg::json::PostgresJsonManager<'a, DATA>;
#[cfg(feature = "pg")]
pub type JsonManagerBuilder<DATA> = pg::json::PostgresJsonManagerBuilder<DATA>;
#[cfg(feature = "pg")]
pub type ToSql = postgres_shared::types::ToSql;
