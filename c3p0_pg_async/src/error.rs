use c3p0_common::error::C3p0Error;

pub fn into_c3p0_error(error: tokio_postgres::error::Error) -> C3p0Error {
    C3p0Error::DbError {
        db: "postgres",
        cause: format!("{}", &error),
        code: error.code().map(|code| code.code().to_owned()),
    }
}

pub fn bb8_into_c3p0_error(error: bb8::RunError<tokio_postgres::error::Error>) -> C3p0Error {
    C3p0Error::DbError {
        db: "postgres",
        cause: format!("{}", &error),
        code: None,
    }
}
