use err_derive::Error;

#[derive(Error, Debug)]
pub enum C3p0Error {
    #[error(display = "PostgresError: [{}]", cause)]
    PostgresError { cause: postgres::error::Error },
    #[error(display = "JsonProcessingError: [{}]", cause)]
    JsonProcessingError { cause: serde_json::error::Error },
    #[error(display = "IteratorError: [{}]", message)]
    IteratorError { message: String },
}

impl From<postgres::error::Error> for C3p0Error {
    fn from(cause: postgres::error::Error) -> Self {
        C3p0Error::PostgresError { cause }
    }
}

impl From<serde_json::error::Error> for C3p0Error {
    fn from(cause: serde_json::error::Error) -> Self {
        C3p0Error::JsonProcessingError { cause }
    }
}
