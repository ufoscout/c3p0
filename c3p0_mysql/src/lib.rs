mod error;
mod json;
mod pool;
mod r2d2;

#[cfg(feature = "migrate")]
mod migrate;

pub use c3p0_common::*;

pub mod mysql {

    pub use crate::json::*;
    pub use crate::pool::*;

    #[cfg(feature = "migrate")]
    pub use crate::migrate::*;

    pub mod r2d2 {
        pub use r2d2::*;
        pub use crate::r2d2::*;
    }

    pub mod driver {
        pub use mysql_client::*;
    }
}
