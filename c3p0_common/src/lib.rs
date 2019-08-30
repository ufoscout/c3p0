pub mod error;
pub mod json;

pub mod pool;
pub mod types;

pub use error::C3p0Error;
pub use json::{
    builder::C3p0JsonBuilder, codec::DefaultJsonCodec, codec::JsonCodec, model::Model,
    model::NewModel, C3p0Json,
};
pub use pool::{C3p0Pool, Connection};

#[cfg(feature = "migrate")]
pub mod migrate;
#[cfg(feature = "migrate")]
pub use migrate::{migration::*, *};
