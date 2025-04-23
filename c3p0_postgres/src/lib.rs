mod common;

mod error;
mod json;
mod pool;
mod query;

pub use common::*;
pub use error::*;
pub use json::*;
pub use pool::*;
pub use query::*;

pub mod tokio_postgres {
    pub use tokio_postgres::*;
}

pub mod deadpool {
    pub use deadpool::*;
    pub mod postgres {
        pub use deadpool_postgres::*;
    }
}

#[cfg(feature = "migrate")]
mod migrate;
#[cfg(feature = "migrate")]
pub use migrate::*;