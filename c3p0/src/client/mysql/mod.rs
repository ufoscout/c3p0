use crate::client::mysql::manager::{MySqlManagerBuilder, MySqlManager};

pub mod error;
pub mod manager;
pub mod pool;

pub type ToSql = mysql_client::Value;

pub type DbManagerBuilder<DATA> = MySqlManagerBuilder<DATA>;
pub type DbManager<'a, DATA> = MySqlManager<'a, DATA>;