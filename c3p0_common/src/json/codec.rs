use crate::error::C3p0Error;
use serde_json::Value;

pub trait JsonCodec<DATA>: Clone
where
    DATA: serde::ser::Serialize + serde::de::DeserializeOwned,
{
    fn from_value(&self, value: Value) -> Result<DATA, C3p0Error>;
    fn to_value(&self, data: &DATA) -> Result<Value, C3p0Error>;
}

#[derive(Clone)]
pub struct DefaultJsonCodec {}

impl<DATA> JsonCodec<DATA> for DefaultJsonCodec
where
    DATA: serde::ser::Serialize + serde::de::DeserializeOwned,
{
    fn from_value(&self, value: Value) -> Result<DATA, C3p0Error> {
        serde_json::from_value::<DATA>(value).map_err(C3p0Error::from)
    }

    fn to_value(&self, data: &DATA) -> Result<Value, C3p0Error> {
        serde_json::to_value(data).map_err(C3p0Error::from)
    }
}
