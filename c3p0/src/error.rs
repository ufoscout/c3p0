use err_derive::Error;

#[derive(Error, Debug)]
pub enum C3p0Error {
    #[error(display = "SqlError: [{}]", cause)]
    SqlError { cause: String },
    #[error(display = "OptimisticLockError: [{}]", message)]
    OptimisticLockError { message: String },
    #[error(display = "JsonProcessingError: [{}]", cause)]
    JsonProcessingError { cause: serde_json::error::Error },
    #[error(display = "IteratorError: [{}]", message)]
    IteratorError { message: String },
}

impl From<serde_json::error::Error> for C3p0Error {
    fn from(cause: serde_json::error::Error) -> Self {
        C3p0Error::JsonProcessingError { cause }
    }
}
