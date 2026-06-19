use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::Stream;
use futures_core::stream::BoxStream;

use crate::C3p0Error;

/// A stream of [`crate::Record`] values produced by [`DbOps::fetch_stream`](crate::DbOps::fetch_stream).
///
/// Wraps sqlx's underlying boxed stream and converts each `sqlx::Error` into a
/// [`C3p0Error`] so callers see the same error type as the rest of the crate.
pub struct RecordStream<'a, T> {
    inner: BoxStream<'a, Result<T, sqlx::Error>>,
}

impl<'a, T> RecordStream<'a, T> {
    pub fn new(inner: BoxStream<'a, Result<T, sqlx::Error>>) -> Self {
        Self { inner }
    }
}

impl<T> Stream for RecordStream<'_, T> {
    type Item = Result<T, C3p0Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // `BoxStream` is `Pin<Box<_>>`, which is `Unpin`, so projecting through
        // `get_mut` is sound.
        let this = self.get_mut();
        this.inner
            .as_mut()
            .poll_next(cx)
            .map(|opt| opt.map(|res| res.map_err(C3p0Error::from)))
    }
}
