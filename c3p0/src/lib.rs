pub mod migrate {
    pub use c3p0_migrate::migration::*;
    pub use c3p0_migrate::*;
}

#[cfg(feature = "mysql")]
pub mod mysql {
    pub use c3p0_pool_mysql::*;
}
#[cfg(feature = "mysql")]
pub use c3p0_pool_mysql::{
    json::C3p0JsonBuilderMysql, json::C3p0JsonMysql, C3p0PoolMysql, MysqlConnection,
};

#[cfg(feature = "pg")]
pub mod pg {
    pub use c3p0_pool_pg::*;
}
#[cfg(feature = "pg")]
pub use c3p0_pool_pg::{json::C3p0JsonBuilderPg, json::C3p0JsonPg, C3p0PoolPg, PgConnection};

#[cfg(feature = "sqlite")]
pub mod sqlite {
    pub use c3p0_pool_sqlite::*;
}
#[cfg(feature = "sqlite")]
pub use c3p0_pool_sqlite::{
    json::C3p0JsonBuilderSqlite, json::C3p0JsonSqlite, C3p0PoolSqlite, SqliteConnection,
};

pub use c3p0_common::error::C3p0Error;
pub use c3p0_common::json::{
    builder::C3p0JsonBuilder, codec::DefaultJsonCodec, codec::JsonCodec, model::Model,
    model::NewModel, C3p0Json,
};
pub use c3p0_common::pool::{C3p0Pool, Connection};
pub use c3p0_common::*;
