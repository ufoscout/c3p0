use c3p0_common::error::C3p0Error;

/// Converts a `deadpool::postgres::PoolError` into a `C3p0Error`.
pub fn deadpool_into_c3p0_error(error: crate::deadpool::postgres::PoolError) -> C3p0Error {
    C3p0Error::PoolError {
        db: "postgres",
        pool: "deadpool",
        cause: format!("{}", &error),
    }
}

/// Converts a `tokio_postgres::Error` into a `C3p0Error`.
pub fn into_c3p0_error(error: tokio_postgres::Error) -> C3p0Error {
    C3p0Error::DbError {
        db: "postgres",
        cause: format!("{}", &error),
        code: error.code().map(|code| code.code().to_owned()),
    }
}
