pub mod error;
pub mod json;

pub mod pool;
pub mod sql;
pub mod types;

pub use error::C3p0Error;
pub use json::{
    builder::C3p0JsonBuilder, codec::DefaultJsonCodec, codec::JsonCodec, model::Model,
    model::NewModel, C3p0Json,
};
pub use pool::{C3p0Pool, SqlConnection};
pub use sql::{ForUpdate, OrderBy};

#[cfg(feature = "async")]
pub use pool::{C3p0PoolAsync, SqlConnectionAsync};

#[cfg(feature = "async")]
pub use json::C3p0JsonAsync;

#[cfg(feature = "migrate")]
pub mod migrate;
