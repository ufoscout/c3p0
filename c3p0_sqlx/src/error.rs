use c3p0_common::C3p0Error;

pub fn into_c3p0_error(error: sqlx::Error) -> C3p0Error {
    C3p0Error::DbError {
        db: "sqlx",
        code: None,
        cause: format!("{}", &error),
    }
}