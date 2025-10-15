use thiserror::Error;

/// An error type for C3p0.
#[derive(Error, Debug)]
pub enum C3p0Error {
    #[error("Error: {cause}")]
    Error { cause: String },
    #[error("OptimisticLockError: {cause}")]
    OptimisticLockError { cause: String },
    #[error("JsonProcessingError: {0:?}")]
    JsonProcessingError (#[from]  serde_json::Error ),
    #[error("JsonProcessingError: {0:?}")]
    SqlxError(#[from] sqlx::Error),
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
