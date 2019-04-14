use crate::error::C3p0Error;
use mysql_client::error::Error;

pub fn into_c3p0_error(error: Error) -> C3p0Error {
    C3p0Error::SqlError {
        cause: format!("{}", error),
    }
}
