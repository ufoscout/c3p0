pub use c3p0_common::*;

#[cfg(any(feature = "in_memory"))]
pub mod in_memory {
    pub use c3p0_in_memory::*;
}

#[cfg(any(feature = "mysql"))]
pub mod mysql {
    pub use c3p0_mysql::*;
}

#[cfg(any(feature = "postgres"))]
pub mod postgres {
    pub use c3p0_postgres::*;
}

#[cfg(any(feature = "sqlx_mysql", feature = "sqlx_postgres", feature = "sqlx_sqlite"))]
pub mod sqlx {
    pub use c3p0_sqlx::*;
}

#[cfg(any(feature = "tidb"))]
pub mod tidb {
    pub use c3p0_mysql::*;
}
