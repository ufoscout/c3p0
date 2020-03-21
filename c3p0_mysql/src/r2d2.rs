use mysql_client::error::Error;
use mysql_client::prelude::Queryable;
use mysql_client::{Conn, Opts, OptsBuilder};
use r2d2;
use std::result::Result;

#[derive(Clone, Debug)]
pub struct MysqlConnectionManager {
    params: Opts,
}

impl MysqlConnectionManager {
    pub fn new(params: OptsBuilder) -> MysqlConnectionManager {
        MysqlConnectionManager {
            params: Opts::from(params),
        }
    }
}

impl r2d2::ManageConnection for MysqlConnectionManager {
    type Connection = Conn;
    type Error = Error;

    fn connect(&self) -> Result<Conn, Error> {
        Conn::new(self.params.clone())
    }

    fn is_valid(&self, conn: &mut Conn) -> Result<(), Error> {
        conn.query_drop("SELECT version()")
    }

    fn has_broken(&self, conn: &mut Conn) -> bool {
        self.is_valid(conn).is_err()
    }
}
