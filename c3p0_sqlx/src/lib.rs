mod common;
pub mod error;

//mod generic;

pub mod sqlx {
    pub use sqlx::*;
}

/*
#[cfg(feature = "migrate")]
mod migrate;
#[cfg(feature = "migrate")]
pub use migrate::*;
*/

#[cfg(any(feature = "mysql"))]
pub mod mysql;

#[cfg(any(feature = "postgres"))]
pub mod postgres;

//#[cfg(any(feature = "sqlite"))]
//pub mod sqlite;