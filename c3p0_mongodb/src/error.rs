use c3p0_common::C3p0Error;

pub fn into_c3p0_error(error: mongodb::error::Error) -> C3p0Error {
    C3p0Error::DbError {
        db: "mongodb",
        cause: format!("{}", &error),
        code: None,
    }
}
