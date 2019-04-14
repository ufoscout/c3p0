use crate::error::C3p0Error;

pub trait C3p0 {
    type ToSql;
    type Connection: Connection<Self::ToSql>;

    fn connection(&self) -> Result<Self::Connection, C3p0Error>;

    fn transaction<T, F: Fn(&Connection<Self::ToSql>) -> Result<T, C3p0Error>>(
        &self,
        tx: F,
    ) -> Result<T, C3p0Error>;
}

pub trait Connection<ToSql> {

    // Add params
    fn execute(&self, sql: &str, params: &[&ToSql]) -> Result<u64, C3p0Error>;

    //fn batch_execute

    //fetch_one
    //fetch_one_option
    //fetch_all


}

