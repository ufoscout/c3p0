use crate::error::C3p0Error;

pub trait C3p0 {
    type Connection: Connection;

    fn connection(&self) -> Result<Self::Connection, C3p0Error>;

    fn transaction<T, F: Fn(&Connection) -> Result<T, C3p0Error>>(
        &self,
        tx: F,
    ) -> Result<T, C3p0Error>;
}

pub trait Connection {

    fn execute(&self, sql: &str) -> Result<u64, C3p0Error>;



}
