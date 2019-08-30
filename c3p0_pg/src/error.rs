use c3p0_common::error::C3p0Error;

pub fn into_c3p0_error(error: postgres::error::Error) -> C3p0Error {
    C3p0Error::SqlError {
        cause: format!("{}", error),
    }
}
