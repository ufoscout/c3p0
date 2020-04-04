use c3p0_common::error::C3p0Error;

pub fn deadpool_into_c3p0_error(error: crate::nio::deadpool::postgres::PoolError) -> C3p0Error {
    C3p0Error::PoolError {
        db: "postgres",
        pool: "deadpool",
        cause: format!("{}", &error),
    }
}
