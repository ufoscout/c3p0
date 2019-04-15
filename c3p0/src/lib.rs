pub mod client;
pub mod error;
pub mod json;
pub mod pool;
pub mod types;

pub mod prelude {
    pub use crate::client::{JsonManager, JsonManagerBuilder};
    pub use crate::error::C3p0Error;
    pub use crate::json::{C3p0Json, C3p0JsonRepository, Model, NewModel, JsonCodec};
    pub use crate::pool::{C3p0, Connection};
}
