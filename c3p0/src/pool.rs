use crate::client::{Row, ToSql};
use crate::error::C3p0Error;

pub trait C3p0 {
    fn connection(&self) -> Result<crate::client::Connection, C3p0Error>;

    fn transaction<T, F: Fn(&crate::client::Transaction) -> Result<T, C3p0Error>>(
        &self,
        tx: F,
    ) -> Result<T, C3p0Error>;
}

pub trait Connection {
    fn execute(&self, sql: &str, params: &[&ToSql]) -> Result<u64, C3p0Error>;

    fn batch_execute(&self, sql: &str) -> Result<(), C3p0Error>;

    fn fetch_one<T, F: Fn(&Row) -> Result<T, C3p0Error>>(
        &self,
        sql: &str,
        params: &[&ToSql],
        mapper: F,
    ) -> Result<T, C3p0Error>;

    fn fetch_one_option<T, F: Fn(&Row) -> Result<T, C3p0Error>>(
        &self,
        sql: &str,
        params: &[&ToSql],
        mapper: F,
    ) -> Result<Option<T>, C3p0Error>;

    //fetch_all

    //count_all_from_table

    //lock_table
}
