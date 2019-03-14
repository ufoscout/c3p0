use crate::error::C3p0Error;
use serde_json::Value;

#[derive(Clone)]
pub struct Codec<DATA>
where
    DATA: serde::ser::Serialize + serde::de::DeserializeOwned,
{
    pub from_value: fn(value: Value) -> Result<DATA, C3p0Error>,
    pub to_value: fn(data: &DATA) -> Result<Value, C3p0Error>,
}

impl<DATA> Default for Codec<DATA>
where
    DATA: serde::ser::Serialize + serde::de::DeserializeOwned,
{
    fn default() -> Self {
        Codec {
            from_value: |value| serde_json::from_value::<DATA>(value).map_err(C3p0Error::from),
            to_value: |data| serde_json::to_value(data).map_err(C3p0Error::from),
        }
    }
}
