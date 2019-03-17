use failure_derive::Fail;

#[derive(Fail, Debug)]
pub enum C3p0MigrateError {
    #[fail(display = "C3p0Error: [{}]", cause)]
    C3p0Error { cause: c3p0_pg::error::C3p0Error },
    #[fail(display = "PostgresError: [{}]", cause)]
    PostgresError { cause: postgres::error::Error },
    #[fail(display = "IteratorError: [{}]", message)]
    IteratorError { message: String },
    #[fail(display = "CorruptedDbMigrationState: [{}]", message)]
    CorruptedDbMigrationState { message: String },
    #[fail(display = "AlteredMigrationSql: [{}]", message)]
    AlteredMigrationSql { message: String },
    #[fail(display = "WrongMigrationSet: [{}]", message)]
    WrongMigrationSet { message: String },
}

impl From<postgres::error::Error> for C3p0MigrateError {
    fn from(cause: postgres::error::Error) -> Self {
        C3p0MigrateError::PostgresError { cause }
    }
}

impl From<c3p0_pg::error::C3p0Error> for C3p0MigrateError {
    fn from(cause: c3p0_pg::error::C3p0Error) -> Self {
        C3p0MigrateError::C3p0Error { cause }
    }
}
