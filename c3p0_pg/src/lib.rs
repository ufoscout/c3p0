#[macro_use]
extern crate rental;

mod error;
mod json;
mod pool;
mod r2d2;

#[cfg(feature = "migrate")]
mod migrate;

pub use c3p0_common::*;

pub mod pg {

    pub use crate::json::*;
    pub use crate::pool::*;

    #[cfg(feature = "migrate")]
    pub use crate::migrate::*;

    pub mod r2d2 {
        pub use r2d2::*;
        //pub use r2d2_postgres::*;
    }
    pub mod driver {
        pub use postgres::*;
    }
}
