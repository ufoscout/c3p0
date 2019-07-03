pub mod client;
pub mod json;

#[cfg(feature = "pg")]
pub use crate::client::pg::{C3p0PgJson, C3p0PgJsonBuilder};
#[cfg(feature = "pg")]
pub use c3p0_pg::{C3p0Pg, C3p0PgBuilder, PgConnection};

pub use crate::json::{codec::JsonCodec, model::Model, model::NewModel, C3p0Json};
pub use c3p0_common::error::C3p0Error;
