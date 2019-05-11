use crate::error::C3p0Error;

pub fn into_c3p0_error(error: rusqlite::Error) -> C3p0Error {
    error.into()
}

impl From<rusqlite::Error> for C3p0Error {
    fn from(error: rusqlite::Error) -> Self {
        C3p0Error::SqlError {
            cause: format!("{}", error),
        }
    }
}
