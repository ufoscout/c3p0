use thiserror::Error;

/// The error type returned by every fallible c3p0 operation.
///
/// Variants split along their source so callers can react differently:
///
/// - [`OptimisticLockError`](Self::OptimisticLockError) is returned only by
///   [`Tx::update`](crate::Tx::update) and [`Tx::delete`](crate::Tx::delete) when the
///   in-memory record's `version` no longer matches the row in the database (i.e. some
///   other writer committed first). It is the *expected* signal that a caller's
///   read-modify-write cycle was outraced and should retry.
/// - [`JsonProcessingError`](Self::JsonProcessingError) wraps `serde_json::Error` and
///   originates from `data` encode/decode at the boundary.
/// - [`SqlxError`](Self::SqlxError) wraps `sqlx::Error` and bubbles up anything the
///   driver reports — connectivity, schema mismatches, missing rows
///   ([`sqlx::Error::RowNotFound`] from `fetch_one*`), constraint violations, etc.
/// - [`Other`](Self::Other) is the catch-all for c3p0-internal errors that don't fit
///   any of the above. It carries a free-form `cause` string and is the variant to
///   construct when surfacing your own validation failures from inside a
///   `pool.transaction(...)` closure.
#[derive(Error, Debug)]
pub enum C3p0Error {
    /// Catch-all for errors raised by c3p0 itself (or by user code inside a
    /// transaction closure) that don't fit one of the more specific variants below.
    /// Prefer the typed variants when applicable; reserve this for genuinely
    /// generic failure modes.
    #[error("Error: {cause}")]
    Other { cause: String },
    /// Returned by [`Tx::update`](crate::Tx::update) / [`Tx::delete`](crate::Tx::delete)
    /// when the row's `version` no longer matches the in-memory record — another
    /// writer committed in between the caller's read and write. Callers typically
    /// re-fetch and retry.
    #[error("OptimisticLockError: {cause}")]
    OptimisticLockError { cause: String },
    /// Wraps a `serde_json::Error` raised while encoding the typed `data` to JSON
    /// before a write, or decoding the JSON back into a typed value after a read.
    #[error("JsonProcessingError: {0:?}")]
    JsonProcessingError(#[from] serde_json::Error),
    /// Wraps a `sqlx::Error` from the underlying driver. Includes connectivity
    /// errors, schema/type mismatches, and `sqlx::Error::RowNotFound` from
    /// `fetch_one*` calls when the requested id does not exist.
    #[error("SqlxError: {0:?}")]
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
