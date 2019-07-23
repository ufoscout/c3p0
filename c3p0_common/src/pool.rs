use crate::error::C3p0Error;
use crate::json::builder::{C3p0JsonBuilder};
use crate::json::C3p0JsonManager;
use crate::json::codec::DefaultJsonCodec;

pub trait C3p0PoolManager: Clone {
    type CONN: Connection;

    fn connection(&self) -> Result<Self::CONN, C3p0Error>;

    fn transaction<T, F: Fn(&Self::CONN) -> Result<T, Box<std::error::Error>>>(
        &self,
        tx: F,
    ) -> Result<T, C3p0Error>;
}

pub trait Connection {
    fn batch_execute(&self, sql: &str) -> Result<(), C3p0Error>;

    /*
    fn execute(&self, sql: &str, params: &[&ToSql]) -> Result<ExecuteResult, C3p0Error>;

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
    */
}
