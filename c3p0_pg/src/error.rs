use failure_derive::Fail;

#[derive(Fail, Debug)]
pub enum C3p0Error {
    #[fail(display = "PostgresError: [{}]", cause)]
    PostgresError { cause: postgres::error::Error },
    #[fail(display = "JsonProcessingError: [{}]", cause)]
    JsonProcessingError { cause: serde_json::error::Error },
    #[fail(display = "IteratorError: [{}]", message)]
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