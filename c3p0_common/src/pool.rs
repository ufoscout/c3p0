use crate::error::C3p0Error;

pub trait C3p0Pool: Clone {
    type CONN;

    fn connection(&self) -> Result<Self::CONN, C3p0Error>;

    fn transaction<T, E: From<C3p0Error>, F: FnOnce(&mut Self::CONN) -> Result<T, E>>(
        &self,
        tx: F,
    ) -> Result<T, E>;
}

pub trait SqlConnection {
    fn batch_execute(&mut self, sql: &str) -> Result<(), C3p0Error>;

    /*
    fn execute(&self, sql: &str, params: &[& dyn ToSql]) -> Result<ExecuteResult, C3p0Error>;

    fn fetch_one_value<T: FromSql>(&self, sql: &str, params: &[& dyn ToSql]) -> Result<T, C3p0Error>;

    fn fetch_one<T, F: Fn(&Row) -> Result<T, Box<dyn std::error::Error>>>(
        &self,
        sql: &str,
        params: &[& dyn ToSql],
        mapper: F,
    ) -> Result<T, C3p0Error>;

    fn fetch_one_option<T, F: Fn(&Row) -> Result<T, Box<dyn std::error::Error>>>(
        &self,
        sql: &str,
        params: &[& dyn ToSql],
        mapper: F,
    ) -> Result<Option<T>, C3p0Error>;

    fn fetch_all<T, F: Fn(&Row) -> Result<T, Box<dyn std::error::Error>>>(
        &self,
        sql: &str,
        params: &[& dyn ToSql],
        mapper: F,
    ) -> Result<Vec<T>, C3p0Error>;

    fn fetch_all_values<T: FromSql>(
        &self,
        sql: &str,
        params: &[& dyn ToSql],
    ) -> Result<Vec<T>, C3p0Error>;

    //count_all_from_table

    //lock_table
    */
}

#[cfg(feature = "async")]
use async_trait::async_trait;
#[cfg(feature = "async")]
use std::future::Future;

#[cfg(feature = "async")]
#[async_trait]
pub trait C3p0PoolAsync: Clone {
    type CONN;

    async fn connection(&self) -> Result<Self::CONN, C3p0Error>;

    async fn transaction<T, E: From<C3p0Error>, F: FnOnce(&Self::CONN) -> dyn Future<Output = Result<T, E>>>(
        &self,
        tx: F,
    ) -> Result<T, E>;
}

#[cfg(feature = "async")]
#[async_trait]
pub trait SqlConnectionAsync {
    async fn batch_execute(&self, sql: &str) -> Result<(), C3p0Error>;

}
