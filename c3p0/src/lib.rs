pub use c3p0_common::*;

#[cfg(feature = "mongodb")]
pub mod mongodb {
    pub use c3p0_mongodb::*;
}
#[cfg(feature = "postgres")]
pub mod postgres {
    pub use c3p0_postgres::*;
}

#[cfg(any(
    feature = "sqlx_mysql",
    feature = "sqlx_postgres",
    feature = "sqlx_sqlite"
))]
pub mod sqlx {
    pub use c3p0_sqlx::*;
}
