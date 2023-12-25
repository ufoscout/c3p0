use serde::{Serialize, de::DeserializeOwned};

pub type VersionType = i32;
pub type EpochMillisType = i64;

pub trait IdType: 'static + Clone + Serialize + DeserializeOwned + Send + Sync + Unpin {}

pub trait DataType: 'static + Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send + Sync + Unpin {}

impl <T: 'static + Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send + Sync + Unpin> DataType for T {}
