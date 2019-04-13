use crate::error::C3p0Error;
use std::ops::Deref;

pub trait C3p0
{
    type Connection: Connection + SqlExecutor;

    fn connection(&self) -> Result<Self::Connection, C3p0Error>;
}

pub trait Connection {
    fn transaction<T, F: Fn(&mut SqlExecutor) -> Result<T, C3p0Error>>(&mut self, tx: F ) -> Result<T, C3p0Error> ;
}

pub trait SqlExecutor {

    fn execute(&mut self, sql: &str) -> Result<u64, C3p0Error>;

}