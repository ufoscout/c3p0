use serde::{Serialize, de::DeserializeOwned};

pub trait Codec<DATA>: Send + Sync + Serialize + DeserializeOwned {
    fn encode(data: DATA) -> Self;

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
