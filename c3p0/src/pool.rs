use crate::error::C3p0Error;
use crate::client::ToSql;

pub trait C3p0 {
    type Connection: Connection;

    fn connection(&self) -> Result<Self::Connection, C3p0Error>;

    fn transaction<T, F: Fn(&Connection) -> Result<T, C3p0Error>>(
        &self,
        tx: F,
    ) -> Result<T, C3p0Error>;
}

pub trait Connection {

    // Add params
    fn execute(&self, sql: &str, params: &[&ToSql]) -> Result<u64, C3p0Error>;

    //fn batch_execute

    //fetch_one
    //fetch_one_option
    //fetch_all


}

