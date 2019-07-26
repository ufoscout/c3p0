pub mod migrate {
    pub use c3p0_migrate::migration::*;
    pub use c3p0_migrate::*;
}

#[cfg(feature = "mysql")]
pub mod mysql {
    pub use c3p0_pool_mysql::*;
}
#[cfg(feature = "mysql")]
pub use c3p0_pool_mysql::{MysqlPoolManager, MysqlConnection, json::MysqlJsonBuilder, json::MysqlJsonManager};


#[cfg(feature = "pg")]
pub mod pg {
    pub use c3p0_pool_pg::*;
}
#[cfg(feature = "pg")]
pub use c3p0_pool_pg::{PgPoolManager, PgConnection, json::PgJsonBuilder, json::PgJsonManager};


#[cfg(feature = "sqlite")]
pub mod sqlite {
    pub use c3p0_pool_sqlite::*;
}
#[cfg(feature = "sqlite")]
pub use c3p0_pool_sqlite::{SqlitePoolManager, SqliteConnection, json::SqliteJsonBuilder, json::SqliteJsonManager};


pub use c3p0_common::json::{codec::JsonCodec, model::Model, model::NewModel, C3p0JsonManager, builder::C3p0JsonBuilder};
pub use c3p0_common::error::C3p0Error;
pub use c3p0_common::pool::{C3p0PoolManager, Connection};
pub use c3p0_common::*;