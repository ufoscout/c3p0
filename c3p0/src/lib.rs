pub mod migrate {
    pub use c3p0_migrate::migration::*;
    pub use c3p0_migrate::*;
}

#[cfg(feature = "mysql")]
pub mod mysql {
    pub use c3p0_pool_mysql::*;
}

#[cfg(feature = "pg")]
pub mod pg {
    pub use c3p0_pool_pg::*;
}

#[cfg(feature = "sqlite")]
pub mod sqlite {
    pub use c3p0_pool_sqlite::*;
}

pub use c3p0_common::json::{codec::JsonCodec, model::Model, model::NewModel, C3p0JsonManager};
pub use c3p0_common::error::C3p0Error;
pub use c3p0_common::pool::{C3p0PoolManager, Connection};
pub use c3p0_common::*;