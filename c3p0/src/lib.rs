pub mod client;
pub mod error;
pub mod json;
pub mod pool;
pub mod types;

pub mod prelude {
    pub use crate::client::{C3p0Builder, JsonManager, JsonManagerBuilder};
    pub use crate::error::C3p0Error;
    pub use crate::json::{codec::JsonCodec, C3p0Json, C3p0JsonRepository, Model, NewModel};
    pub use crate::pool::{C3p0, Connection};
}
