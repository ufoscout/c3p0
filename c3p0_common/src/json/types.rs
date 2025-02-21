use std::fmt::Debug;

use serde::{Serialize, de::DeserializeOwned};

pub type VersionType = u32;
pub type EpochMillisType = i64;

pub trait DataType:
    'static + Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send + Sync + Unpin
{
}

impl<T: 'static + Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send + Sync + Unpin>
    DataType for T
{
}

pub trait IdType:
    'static
    + Clone
    + Serialize
    + DeserializeOwned
    + Debug
    + Send
    + Sync
    + Unpin
    + PartialEq
    + Eq
    + PartialOrd
    + Ord
{
}

impl<
    T: 'static
        + Clone
        + Serialize
        + DeserializeOwned
        + Debug
        + Send
        + Sync
        + Unpin
        + PartialEq
        + Eq
        + PartialOrd
        + Ord,
> IdType for T
{
}
