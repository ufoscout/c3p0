#![doc = include_utils::include_md!("README.md")]

pub use c3p0_common::*;

#[cfg(feature = "postgres")]
pub mod postgres {
    pub use c3p0_postgres::*;
}

#[cfg(any(
    feature = "sqlx",
    feature = "sqlx_mysql",
    feature = "sqlx_postgres",
    feature = "sqlx_sqlite"
))]
pub mod sqlx {
    pub use c3p0_sqlx::*;
}
