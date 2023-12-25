use crate::{error::C3p0Error, DataType};
use serde_json::Value;

pub trait JsonCodec<Data: DataType>: Clone + Send + Sync {
    fn data_from_value(&self, value: Value) -> Result<Data, C3p0Error>;
    fn data_to_value(&self, data: &Data) -> Result<Value, C3p0Error>;
}

#[derive(Clone, Default)]
pub struct DefaultJsonCodec {}

impl<Data: DataType> JsonCodec<Data> for DefaultJsonCodec {
    fn data_from_value(&self, value: Value) -> Result<Data, C3p0Error> {
        serde_json::from_value::<Data>(value).map_err(C3p0Error::from)
    }

    fn data_to_value(&self, data: &Data) -> Result<Value, C3p0Error> {
        serde_json::to_value(data).map_err(C3p0Error::from)
    }
}
