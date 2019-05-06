use err_derive::Error;

#[derive(Error, Debug)]
pub enum C3p0MigrateError {
    #[error(display = "C3p0Error: [{}]", cause)]
    C3p0Error { cause: c3p0::error::C3p0Error },
    #[error(display = "IteratorError: [{}]", message)]
    IteratorError { message: String },
    #[error(display = "CorruptedDbMigrationState: [{}]", message)]
    CorruptedDbMigrationState { message: String },
    #[error(display = "AlteredMigrationSql: [{}]", message)]
    AlteredMigrationSql { message: String },
    #[error(display = "WrongMigrationSet: [{}]", message)]
    WrongMigrationSet { message: String },
    #[error(display = "FileSystemError: [{}]", message)]
    FileSystemError { message: String },
}

impl From<c3p0::error::C3p0Error> for C3p0MigrateError {
    fn from(cause: c3p0::error::C3p0Error) -> Self {
        C3p0MigrateError::C3p0Error { cause }
    }
}
