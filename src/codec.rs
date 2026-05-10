use serde::{Serialize, de::DeserializeOwned};

/// Lossless converter between a [`DataType`](crate::DataType)'s in-memory `DATA` and
/// the shape that gets serialised into the `data` JSON column.
pub trait Codec<DATA>: Send + Sync + Serialize + DeserializeOwned {
    /// Convert an in-memory value into the on-disk shape just before writing.
    fn encode(data: DATA) -> Self;

    /// Convert an on-disk value back into the in-memory shape just after reading.
    fn decode(data: Self) -> DATA;
}

impl<T: Send + Sync + Serialize + DeserializeOwned> Codec<T> for T {
    #[inline(always)]
    fn encode(data: T) -> Self {
        data
    }

    #[inline(always)]
    fn decode(data: Self) -> T {
        data
    }
}
