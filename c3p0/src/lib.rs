pub use c3p0_common::*;

#[cfg(feature = "in_memory")]
pub mod in_memory {
    pub use c3p0_in_memory::*;
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
