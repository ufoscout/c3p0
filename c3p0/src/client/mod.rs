#[cfg(feature = "mysql")]
mod mysql;

#[cfg(feature = "mysql")]
pub type DbManager<'a, DATA> = mysql::manager::MySqlManager<'a, DATA>;
#[cfg(feature = "mysql")]
pub type DbManagerBuilder<DATA> = mysql::manager::MySqlManagerBuilder<DATA>;
#[cfg(feature = "mysql")]
pub type ToSql = mysql_client::Value;


#[cfg(feature = "pg")]
mod pg;

#[cfg(feature = "pg")]
pub type DbManager<'a, DATA> = pg::manager::PostgresManager<'a, DATA>;
#[cfg(feature = "pg")]
pub type DbManagerBuilder<DATA> = pg::manager::PostgresManagerBuilder<DATA>;
#[cfg(feature = "pg")]
pub type ToSql = postgres_shared::types::ToSql;


