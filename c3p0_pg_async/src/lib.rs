//mod bb8;
mod error;
mod json;
mod pool;

#[cfg(feature = "migrate")]
mod migrate;

pub mod pg_async {

    //  pub use crate::bb8::*;
    pub use crate::json::*;
    pub use crate::pool::*;

    #[cfg(feature = "migrate")]
    pub use crate::migrate::*;

    pub mod bb8 {
        pub use bb8::*;
        pub use bb8_postgres::*;
    }
    pub mod driver {
        pub use tokio_postgres::*;
    }
}
