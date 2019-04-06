use err_derive::Error;
use mysql::error::Error as MyError;

#[derive(Error, Debug)]
pub enum C3p0Error {
    #[error(display = "DbError: [{}]", cause)]
    DbError { cause: String },
    #[error(display = "JsonProcessingError: [{}]", cause)]
    JsonProcessingError { cause: serde_json::error::Error },
    #[error(display = "IteratorError: [{}]", message)]
    IteratorError { message: String },
}

impl From<MyError> for C3p0Error {
    fn from(cause: MyError) -> Self {
        C3p0Error::DbError { cause: format!("{}", cause) }
    }
}

impl From<serde_json::error::Error> for C3p0Error {
    fn from(cause: serde_json::error::Error) -> Self {
        C3p0Error::JsonProcessingError { cause }
    }
}
