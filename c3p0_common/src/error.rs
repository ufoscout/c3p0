use thiserror::Error;

/// An error type for C3p0.
#[derive(Error, Debug)]
pub enum C3p0Error {
    #[error("InternalError: {cause}")]
    InternalError { cause: String },
    #[error("DbError: {db} - {cause} - code: {code:?}")]
    DbError {
        db: &'static str,
        cause: String,
        code: Option<String>,
    },
    #[error("RowMapperError: {cause}")]
    RowMapperError { cause: String },
    #[error("OptimisticLockError: {cause}")]
    OptimisticLockError { cause: String },
    #[error("JsonProcessingError: {cause:?}")]
    JsonProcessingError { cause: serde_json::error::Error },
    #[error("InternalError: {cause}")]
    IteratorError { cause: String },
    #[error("PoolError: {db} - {pool} - {cause}")]
    PoolError {
        db: &'static str,
        pool: &'static str,
        cause: String,
    },
    #[error("ResultNotFoundError")]
    ResultNotFoundError,
    #[error("TransactionError: {cause:?}")]
    TransactionError {
        cause: Box<dyn std::error::Error + Send + Sync>,
    },
    #[error("IoError: {cause}")]
    IoError { cause: String },
    #[error("MigrationError: {cause}. Source: {source:?}")]
    MigrationError {
        cause: String,
        source: Box<C3p0Error>,
    },
    #[error("CorruptedDbMigrationState: {cause}")]
    CorruptedDbMigrationState { cause: String },
    #[error("OperationNotSupported: {cause}")]
    OperationNotSupported { cause: String },
}

impl From<serde_json::error::Error> for C3p0Error {
    fn from(err: serde_json::Error) -> Self {
        C3p0Error::JsonProcessingError { cause: err }
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
