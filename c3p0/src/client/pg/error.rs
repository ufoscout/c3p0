use crate::error::C3p0Error;

pub fn into_c3p0_error(error: postgres::error::Error) -> C3p0Error {
    error.into()
}

impl From<postgres::error::Error> for C3p0Error {
    fn from(error: postgres::error::Error) -> Self {
        C3p0Error::SqlError {
            cause: format!("{}", error),
        }
    }
}
