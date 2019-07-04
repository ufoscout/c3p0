use err_derive::Error;

#[derive(Error, Debug)]
pub enum C3p0Error {
    #[error(display = "SqlError: [{}]", cause)]
    SqlError { cause: String },
    #[error(display = "RowMapperError: [{}]", cause)]
    RowMapperError { cause: String },
    #[error(display = "OptimisticLockError: [{}]", message)]
    OptimisticLockError { message: String },
    #[error(display = "JsonProcessingError: [{}]", cause)]
    JsonProcessingError { cause: serde_json::error::Error },
    #[error(display = "IteratorError: [{}]", message)]
    IteratorError { message: String },
    #[error(display = "PoolError: [{}]", cause)]
    PoolError { cause: String },
    #[error(display = "ResultNotFoundError: Expected one result but found zero.")]
    ResultNotFoundError,
    #[error(display = "TransactionError: [{}]", cause)]
    TransactionError { cause: Box<std::error::Error> },
    #[error(display = "CorruptedDbMigrationState: [{}]", message)]
    CorruptedDbMigrationState { message: String },
    #[error(display = "AlteredMigrationSql: [{}]", message)]
    AlteredMigrationSql { message: String },
    #[error(display = "WrongMigrationSet: [{}]", message)]
    WrongMigrationSet { message: String },
    #[error(display = "FileSystemError: [{}]", message)]
    FileSystemError { message: String },
}

impl From<serde_json::error::Error> for C3p0Error {
    fn from(cause: serde_json::error::Error) -> Self {
        C3p0Error::JsonProcessingError { cause }
    }
}
