use crate::{DataType, error::C3p0Error};
use serde_json::Value;

/// A JSON codec for a specific data type.
pub trait JsonCodec<Data: DataType>: Clone + Send + Sync {
    /// Returns the default codec.
    fn default() -> DefaultJsonCodec {
        DefaultJsonCodec {}
    }

    /// Deserialize a JSON value into a `Data` value.
    fn data_from_value(&self, value: Value) -> Result<Data, C3p0Error>;

    /// Serialize a `Data` value into a JSON value.
    fn data_to_value(&self, data: &Data) -> Result<Value, C3p0Error>;
}

/// Default JSON codec.
#[derive(Clone, Default)]
pub struct DefaultJsonCodec {}

impl<Data: DataType> JsonCodec<Data> for DefaultJsonCodec {
    /// Deserialize a JSON value into a `Data` value.
    fn data_from_value(&self, value: Value) -> Result<Data, C3p0Error> {
        serde_json::from_value::<Data>(value).map_err(C3p0Error::from)
    }

    /// Serialize a `Data` value into a JSON value.
    fn data_to_value(&self, data: &Data) -> Result<Value, C3p0Error> {
        serde_json::to_value(data).map_err(C3p0Error::from)
    }
}
