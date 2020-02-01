//mod bb8;
mod error;
//mod json;
mod pool;

pub use c3p0_common::*;

pub mod pg {

    //  pub use crate::bb8::*;
    //  pub use crate::json::*;
    ///  pub use crate::pool::*;

    pub mod bb8 {
        pub use bb8::*;
        pub use bb8_postgres::*;
    }
    pub mod driver {
        pub use tokio_postgres::*;
    }
}
