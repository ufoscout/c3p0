#[cfg(feature = "mysql")]
mod mysql;

#[cfg(feature = "mysql")]
pub type C3p0Builder = mysql::pool::C3p0MySqlBuilder;
#[cfg(feature = "mysql")]
pub type ToSql = mysql::pool::ToSql;
#[cfg(feature = "mysql")]
pub type JsonManager<'a, DATA> = mysql::json::MySqlJsonManager<'a, DATA>;
#[cfg(feature = "mysql")]
pub type JsonManagerBuilder<DATA> = mysql::json::MySqlJsonManagerBuilder<DATA>;

#[cfg(feature = "pg")]
mod pg;

#[cfg(feature = "pg")]
pub type C3p0Builder = pg::pool::C3p0PgBuilder;
#[cfg(feature = "pg")]
pub type JsonManager<'a, DATA> = pg::json::PostgresJsonManager<'a, DATA>;
#[cfg(feature = "pg")]
pub type JsonManagerBuilder<DATA> = pg::json::PostgresJsonManagerBuilder<DATA>;
#[cfg(feature = "pg")]
pub type ToSql = pg::pool::ToSql;

pub const NO_PARAMS: &[&ToSql] = &[];