use std::fmt::Debug;

use serde::{Serialize, de::DeserializeOwned};

pub type VersionType = i32;
pub type EpochMillisType = i64;

pub trait DataType: 'static + Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send + Sync + Unpin {}

impl <T: 'static + Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send + Sync + Unpin> DataType for T {}


pub trait IdType: 'static + Clone + Serialize + DeserializeOwned + Debug + Send + Sync + Unpin
+ crate::database::mongodb::types::MaybeIntoBson
{}

impl <T: 'static + Clone + Serialize + DeserializeOwned + Debug + Send + Sync + Unpin 
+ crate::database::mongodb::types::MaybeIntoBson
> IdType for T {}

// macro_rules! id_type_trait {
//     ($($bounds:ident),*) => {
//         pub trait IdType: 'static + Clone + Serialize + DeserializeOwned + Debug + Send + Sync + Unpin
//         where $(Self: $bounds),* {}

//         impl <T: 'static + Clone + Serialize + DeserializeOwned + Debug + Send + Sync + Unpin > IdType for T
//         where $(Self: $bounds),* {}
//     };
// }

// #[cfg(not(feature = "thread_safe"))]
// id_type_trait!();
// #[cfg(feature = "thread_safe")]
// id_type_trait!(Send, Sync);
