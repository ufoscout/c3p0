use c3p0_common::error::C3p0Error;

pub fn into_c3p0_error(error: rusqlite::Error) -> C3p0Error {
    C3p0Error::DbError {
        db: "sqlite",
        cause: format!("{}", error),
        code: None,
    }
}
