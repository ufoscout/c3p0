use crate::client::{Row, ToSql};
use crate::error::C3p0Error;

#[cfg(feature = "mysql")]
use mysql_client::prelude::FromValue as FromSql;
#[cfg(feature = "pg")]
use postgres::types::FromSql;

pub trait C3p0Base: Clone {
    fn connection(&self) -> Result<crate::client::Connection, C3p0Error>;

    fn transaction<T, F: Fn(&crate::client::Connection) -> Result<T, Box<std::error::Error>>>(
        &self,
        tx: F,
    ) -> Result<T, C3p0Error>;
}

pub trait ConnectionBase {
    fn execute(&self, sql: &str, params: &[&ToSql]) -> Result<u64, C3p0Error>;

    fn batch_execute(&self, sql: &str) -> Result<(), C3p0Error>;

    fn fetch_one_value<T: FromSql>(&self, sql: &str, params: &[&ToSql]) -> Result<T, C3p0Error>;

    fn fetch_one<T, F: Fn(&Row) -> Result<T, Box<std::error::Error>>>(
        &self,
        sql: &str,
        params: &[&ToSql],
        mapper: F,
    ) -> Result<T, C3p0Error>;

    fn fetch_one_option<T, F: Fn(&Row) -> Result<T, Box<std::error::Error>>>(
        &self,
        sql: &str,
        params: &[&ToSql],
        mapper: F,
    ) -> Result<Option<T>, C3p0Error>;

    fn fetch_all<T, F: Fn(&Row) -> Result<T, Box<std::error::Error>>>(
        &self,
        sql: &str,
        params: &[&ToSql],
        mapper: F,
    ) -> Result<Vec<T>, C3p0Error>;

    fn fetch_all_values<T: FromSql>(
        &self,
        sql: &str,
        params: &[&ToSql],
    ) -> Result<Vec<T>, C3p0Error>;

    //count_all_from_table

    //lock_table
}
