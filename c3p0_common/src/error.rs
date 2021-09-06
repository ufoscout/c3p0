use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum C3p0Error {
    InternalError { cause: String },
    DbError {
        db: &'static str,
        cause: String,
        code: Option<String>,
    },
    RowMapperError { cause: String },
    OptimisticLockError { message: String },
    JsonProcessingError {
        source: serde_json::error::Error,
    },
    IteratorError { message: String },
    PoolError {
        db: &'static str,
        pool: &'static str,
        cause: String,
    },
    ResultNotFoundError,
    TransactionError {
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    CorruptedDbMigrationState { message: String },
    AlteredMigrationSql { message: String },
    WrongMigrationSet { message: String },
    IoError { message: String },
    MigrationError {
        message: String,
        source: Box<C3p0Error>,
    },
}

impl Display for C3p0Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            C3p0Error::InternalError { cause } => write!(f, "InternalError: [{}]", cause),
            C3p0Error::DbError {db, cause, code} => write!(f, "DbError. DB: {}. DB specific error code: [{:?}]. Msg: {}", db, code, cause),
            C3p0Error::RowMapperError { cause } => write!(f, "RowMapperError: [{}]", cause),
            C3p0Error::OptimisticLockError { message } => write!(f, "OptimisticLockError: [{}]", message),
            C3p0Error::JsonProcessingError {..} => write!(f, "JsonProcessingError"),
            C3p0Error::IteratorError { message } => write!(f, "IteratorError: [{}]", message),
            C3p0Error::PoolError {db, pool, cause} => write!(f, "PoolError: pool [{}] for [{}] returned error: [{}]", pool, db, cause),
            C3p0Error::ResultNotFoundError => write!(f, "ResultNotFoundError: Expected one result but found zero."),
            C3p0Error::TransactionError { .. } => write!(f, "TransactionError"),
            C3p0Error::CorruptedDbMigrationState { message } => write!(f, "CorruptedDbMigrationState: [{}]", message),
            C3p0Error::AlteredMigrationSql { message } => write!(f, "AlteredMigrationSql: [{}]", message),
            C3p0Error::WrongMigrationSet { message } => write!(f, "WrongMigrationSet: [{}]", message),
            C3p0Error::IoError { message } => write!(f, "IoError: [{}]", message),
            C3p0Error::MigrationError {message, source: _} => write!(f, "MigrationError: [{}]", message),
        }
    }
}

impl From<serde_json::error::Error> for C3p0Error {
    fn from(err: serde_json::Error) -> Self {
        C3p0Error::JsonProcessingError { source: err }
    }
}

impl Error for C3p0Error {

    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            C3p0Error::InternalError { .. } |
            C3p0Error::DbError { .. } |
            C3p0Error::RowMapperError { .. } |
            C3p0Error::OptimisticLockError { .. } |
            C3p0Error::IteratorError { .. } |
            C3p0Error::PoolError { ..} |
            C3p0Error::ResultNotFoundError |
            C3p0Error::CorruptedDbMigrationState { .. } |
            C3p0Error::AlteredMigrationSql { .. } |
            C3p0Error::WrongMigrationSet { .. } |
            C3p0Error::IoError { .. } => None,
            C3p0Error::JsonProcessingError { source, .. } => Some(source),
            C3p0Error::MigrationError { source, .. } => Some(source.as_ref()),
            C3p0Error::TransactionError { source} => Some(source.as_ref()),
        }
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

    #[test]
    fn test_print_error() {
        let err = C3p0Error::MigrationError {
            message: "BBBBBBBBBb".to_owned(),
            source: Box::new(C3p0Error::AlteredMigrationSql {
                message: "AAAAAAAAAAA".to_owned(),
            }),
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
            }),
        })?
    }
}
