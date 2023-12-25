use std::fmt::Debug;

use serde::{Serialize, de::DeserializeOwned};

pub type VersionType = i32;
pub type EpochMillisType = i64;

pub trait IdType: 'static + Clone + Serialize + DeserializeOwned + Debug + Send + Sync + Unpin {}

impl <T: 'static + Clone + Serialize + DeserializeOwned + Debug + Send + Sync + Unpin> IdType for T {}

pub trait DataType: 'static + Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send + Sync + Unpin {}

impl <T: 'static + Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send + Sync + Unpin> DataType for T {}
