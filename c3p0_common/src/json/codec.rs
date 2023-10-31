use crate::error::C3p0Error;
use serde_json::Value;

pub trait JsonCodec<Data>: Clone + Send + Sync
where
    Data: serde::ser::Serialize + serde::de::DeserializeOwned + Send,
{
    fn data_from_value(&self, value: Value) -> Result<Data, C3p0Error>;
    fn data_to_value(&self, data: &Data) -> Result<Value, C3p0Error>;
}

#[derive(Clone, Default)]
pub struct DefaultJsonCodec {}

impl<Data> JsonCodec<Data> for DefaultJsonCodec
where
    Data: serde::ser::Serialize + serde::de::DeserializeOwned + Send,
{
    fn data_from_value(&self, value: Value) -> Result<Data, C3p0Error> {
        serde_json::from_value::<Data>(value).map_err(C3p0Error::from)
    }

    fn data_to_value(&self, data: &Data) -> Result<Value, C3p0Error> {
        serde_json::to_value(data).map_err(C3p0Error::from)
    }
}
