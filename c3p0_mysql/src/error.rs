use c3p0_common::error::C3p0Error;
use mysql_client::error::Error;

pub fn into_c3p0_error(error: Error) -> C3p0Error {
    C3p0Error::SqlError {
        cause: format!("{}", error),
    }
}

/*
impl From<std::option::NoneError> for C3p0Error {
    fn from(error: std::option::NoneError) -> Self {
        C3p0Error::SqlError {
            cause: format!("Expected a value, found none"),
        }
    }
}
*/

/*
impl From<mysql_common::value::convert::FromValueError> for C3p0Error {
    fn from(error: mysql_common::value::convert::FromValueError) -> Self {
        C3p0Error::SqlError {
            cause: format!("{}", error),
        }
    }
}
*/
