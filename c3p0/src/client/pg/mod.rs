use crate::client::pg::manager::{PostgresManagerBuilder, PostgresManager};

pub mod error;
pub mod manager;
pub mod pool;

pub type ToSql = postgres_shared::types::ToSql;

pub type DbManagerBuilder<DATA> = PostgresManagerBuilder<DATA>;
pub type DbManager<'a, DATA> = PostgresManager<'a, DATA>;