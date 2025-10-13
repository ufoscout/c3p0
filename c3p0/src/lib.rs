#![doc = include_utils::include_md!("README.md")]

pub use c3p0_common::*;

#[cfg(any(
    feature = "sqlx",
    feature = "sqlx_mysql",
    feature = "sqlx_postgres",
    feature = "sqlx_sqlite"
))]
pub mod sqlx {
    pub use c3p0_sqlx::*;
}
