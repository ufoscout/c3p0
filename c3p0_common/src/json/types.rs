use std::fmt::Debug;

use serde::{Serialize, de::DeserializeOwned};

/// A type alias for the version of a model.
pub type VersionType = u32;

/// A type alias for the epoch millis of a model.
pub type EpochMillisType = i64;

/// A trait for a data type.
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
