#![feature(async_fn_in_trait)]
#![feature(return_position_impl_trait_in_trait)]

mod common;
mod error;
mod json;
mod pool;

pub use common::*;
pub use error::*;
pub use json::*;
pub use pool::*;

pub mod tokio_postgres {
    pub use tokio_postgres::*;
}

pub mod deadpool {
    pub use deadpool::*;
    pub mod postgres {
        pub use deadpool_postgres::*;
    }
}

// #[cfg(feature = "migrate")]
// mod migrate;
// #[cfg(feature = "migrate")]
// pub use migrate::*;
