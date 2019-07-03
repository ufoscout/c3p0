pub mod client;
pub mod json;

pub use crate::json::{codec::JsonCodec, model::Model, model::NewModel, C3p0Json};
pub use c3p0_common::error::C3p0Error;
