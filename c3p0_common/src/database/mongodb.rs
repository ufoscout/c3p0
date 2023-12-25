

#[cfg(feature = "mongodb")]
pub mod types {

    use crate::C3p0Error;

    impl From<mongodb::error::Error> for C3p0Error {
        fn from(err: mongodb::error::Error) -> Self {
            C3p0Error::DbError {
                db: "mongodb",
                cause: format!("{}", &err),
                code: None,
            }
        }
    }

    pub trait MaybeMongodb: Into<mongodb::bson::Bson> {}
    impl<T: Into<mongodb::bson::Bson>> MaybeMongodb for T {}
}

#[cfg(not(feature = "mongodb"))]
pub mod types {
    pub trait MaybeMongodb {}
    impl<T> MaybeMongodb for T where T: ?Sized {}
}