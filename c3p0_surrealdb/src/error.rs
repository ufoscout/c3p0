use c3p0_common::error::C3p0Error;

pub fn deadpool_into_c3p0_error(error: crate::deadpool::managed::PoolError<surrealdb::Error>) -> C3p0Error {
    C3p0Error::PoolError {
        db: "surrealdb",
        pool: "deadpool",
        cause: format!("{}", &error),
    }
}

pub fn into_c3p0_error(error: surrealdb::Error) -> C3p0Error {
    C3p0Error::DbError {
        db: "surrealdb",
        code: None,
        cause: format!("{}", &error),
    }
}
