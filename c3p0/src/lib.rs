pub use c3p0_common::*;

#[cfg(any(feature = "in_memory_blocking"))]
pub mod in_memory {
    pub use c3p0_in_memory::*;
}

#[cfg(any(feature = "mysql_blocking"))]
pub mod mysql {
    pub use c3p0_mysql::*;
}

#[cfg(any(feature = "pg", feature = "pg_blocking"))]
pub mod pg {
    pub use c3p0_pg::*;
}

#[cfg(any(feature = "sqlite_blocking"))]
pub mod sqlite {
    pub use c3p0_sqlite::*;
}