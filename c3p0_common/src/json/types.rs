use std::fmt::Debug;

use serde::{de::DeserializeOwned, Serialize};

pub type VersionType = u32;
pub type EpochMillisType = i64;

pub trait DataType:
    'static + Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send + Sync + Unpin
{
}

impl<
        T: 'static + Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send + Sync + Unpin,
    > DataType for T
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
    + crate::database::mongodb::types::MaybeMongodb
    + crate::database::postgres::types::MaybePostgres
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
            + crate::database::mongodb::types::MaybeMongodb
            + crate::database::postgres::types::MaybePostgres,
    > IdType for T
{
}
