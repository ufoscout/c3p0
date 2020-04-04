use thiserror::Error;

#[derive(Error, Debug)]
pub enum C3p0Error {
    #[error("InternalError: [{cause}]")]
    InternalError { cause: String },
    #[error("DbError. DB: {db}. DB specific error code: [{code:?}]. Msg: {cause}")]
    DbError {
        db: &'static str,
        cause: String,
        code: Option<String>,
    },
    #[error("RowMapperError: [{cause}]")]
    RowMapperError { cause: String },
    #[error("OptimisticLockError: [{message}]")]
    OptimisticLockError { message: String },
    #[error("JsonProcessingError: [{cause}]")]
    JsonProcessingError { cause: serde_json::error::Error },
    #[error("IteratorError: [{message}]")]
    IteratorError { message: String },
    #[error("PoolError: pool [{pool}] for [{db}] returned error: [{cause}]")]
    PoolError {
        db: &'static str,
        pool: &'static str,
        cause: String,
    },
    #[error("ResultNotFoundError: Expected one result but found zero.")]
    ResultNotFoundError,
    #[error("TransactionError: [{cause}]")]
    TransactionError {
        cause: Box<dyn std::error::Error + Send + Sync>,
    },
    #[error("CorruptedDbMigrationState: [{message}]")]
    CorruptedDbMigrationState { message: String },
    #[error("AlteredMigrationSql: [{message}]")]
    AlteredMigrationSql { message: String },
    #[error("WrongMigrationSet: [{message}]")]
    WrongMigrationSet { message: String },
    #[error("FileSystemError: [{message}]")]
    FileSystemError { message: String },
    #[error("MigrationError: [{message}]. Cause: [{cause}]")]
    MigrationError {
        message: String,
        cause: Box<C3p0Error>,
    },
}

impl From<serde_json::error::Error> for C3p0Error {
    fn from(cause: serde_json::error::Error) -> Self {
        C3p0Error::JsonProcessingError { cause }
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use static_assertions::*;

    #[test]
    fn error_should_be_send_and_sync() {
        assert_impl_all!(C3p0Error: Send, Sync);
    }
}
