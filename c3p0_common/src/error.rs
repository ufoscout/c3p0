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
    #[error("JsonProcessingError")]
    JsonProcessingError {
        #[from]
        #[source]
        source: serde_json::error::Error
    },
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
    #[error("TransactionError")]
    TransactionError {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    #[error("CorruptedDbMigrationState: [{message}]")]
    CorruptedDbMigrationState { message: String },
    #[error("AlteredMigrationSql: [{message}]")]
    AlteredMigrationSql { message: String },
    #[error("WrongMigrationSet: [{message}]")]
    WrongMigrationSet { message: String },
    #[error("IoError: [{message}]")]
    IoError { message: String },
    #[error("MigrationError: [{message}]")]
    MigrationError {
        message: String,
        #[source]
        source: Box<C3p0Error>,
    },
}

#[cfg(test)]
mod test {

    use super::*;
    use static_assertions::*;

    #[test]
    fn error_should_be_send_and_sync() {
        assert_impl_all!(C3p0Error: Send, Sync);
    }

    #[test]
    fn test_print_error() {
        let err = C3p0Error::MigrationError {
            message: "BBBBBBBBBb".to_owned(),
            source: Box::new(C3p0Error::AlteredMigrationSql {
                message: "AAAAAAAAAAA".to_owned(),
            })
        };

        println!("display: {}", err);
        println!("debug: {:?}", err);

        let any_err = fn1();

        // To see the full backtrace, use nightly rust:
        // > RUST_BACKTRACE=1 cargo +nightly test test_print_error -- --nocapture
        println!("debug: {:?}", any_err);
    }

    fn fn1() -> anyhow::Result<()> {
        Err(C3p0Error::MigrationError {
            message: "BBBBBBBBBb".to_owned(),
            source: Box::new(C3p0Error::AlteredMigrationSql {
                message: "AAAAAAAAAAA".to_owned(),
            })
        })?
    }
}
