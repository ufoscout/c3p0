#[cfg(feature = "postgres")]
pub mod types {

    use tokio_postgres::types::{FromSqlOwned, ToSql};

    use crate::C3p0Error;

    impl From<tokio_postgres::Error> for C3p0Error {
        fn from(error: tokio_postgres::Error) -> Self {
            C3p0Error::DbError {
                db: "postgres",
                cause: format!("{}", &error),
                code: error.code().map(|code| code.code().to_owned()),
            }
        }
    }

    pub trait MaybePostgres: FromSqlOwned + ToSql {}
    impl<T: FromSqlOwned + ToSql> MaybePostgres for T {}
}

#[cfg(not(feature = "postgres"))]
pub mod types {
    pub trait MaybePostgres {}
    impl<T> MaybePostgres for T where T: ?Sized {}
}
