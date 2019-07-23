pub use c3p0_json::*;

pub mod migrate {
    pub use c3p0_migrate::migration::*;
    pub use c3p0_migrate::*;
}

#[cfg(feature = "mysql")]
#[derive(Clone)]
pub struct C3p0Builder {
    pool: c3p0_json::mysql::r2d2::Pool<c3p0_json::mysql::r2d2::MysqlConnectionManager>,
}

#[cfg(feature = "mysql")]
impl C3p0Builder {
    pub fn new(
        pool: c3p0_json::mysql::r2d2::Pool<c3p0_json::mysql::r2d2::MysqlConnectionManager>,
    ) -> Self {
        C3p0Builder { pool }
    }

    pub fn pool(&self) -> c3p0_pool_mysql::C3p0Mysql {
        c3p0_pool_mysql::C3p0MysqlBuilder::build(self.pool.clone())
    }

    pub fn json<
        T: Into<String>,
        DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
    >(
        &self,
        table_name: T,
    ) -> c3p0_json::C3p0MysqlJsonBuilder<DATA, c3p0_json::json::codec::DefaultJsonCodec> {
        c3p0_json::C3p0MysqlJsonBuilder::new(table_name)
    }

    pub fn migrate(&self) -> c3p0_migrate::C3p0MigrateBuilder<c3p0_pool_mysql::C3p0Mysql> {
        c3p0_migrate::C3p0MigrateBuilder::new(self.pool())
    }
}

#[cfg(feature = "pg")]
#[derive(Clone)]
pub struct C3p0Builder {
    pool: c3p0_json::pg::r2d2::Pool<c3p0_json::pg::r2d2::PostgresConnectionManager>,
}

#[cfg(feature = "pg")]
impl C3p0Builder {
    pub fn new(
        pool: c3p0_json::pg::r2d2::Pool<c3p0_json::pg::r2d2::PostgresConnectionManager>,
    ) -> Self {
        C3p0Builder { pool }
    }

    pub fn pool(&self) -> c3p0_pool_pg::C3p0Pg {
        c3p0_pool_pg::C3p0PgBuilder::build(self.pool.clone())
    }

    pub fn json<
        T: Into<String>,
        DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
    >(
        &self,
        table_name: T,
    ) -> c3p0_json::C3p0PgJsonBuilder<DATA, c3p0_json::json::codec::DefaultJsonCodec> {
        c3p0_json::C3p0PgJsonBuilder::new(table_name)
    }

    pub fn migrate(&self) -> c3p0_migrate::C3p0MigrateBuilder<c3p0_pool_pg::C3p0Pg> {
        c3p0_migrate::C3p0MigrateBuilder::new(self.pool())
    }
}

#[cfg(feature = "sqlite")]
#[derive(Clone)]
pub struct C3p0Builder {
    pool: c3p0_json::sqlite::r2d2::Pool<c3p0_json::sqlite::r2d2::SqliteConnectionManager>,
}

#[cfg(feature = "sqlite")]
impl C3p0Builder {
    pub fn new(
        pool: c3p0_json::sqlite::r2d2::Pool<c3p0_json::sqlite::r2d2::SqliteConnectionManager>,
    ) -> Self {
        C3p0Builder { pool }
    }

    pub fn pool(&self) -> c3p0_pool_sqlite::C3p0Sqlite {
        c3p0_pool_sqlite::C3p0SqliteBuilder::build(self.pool.clone())
    }

    pub fn json<
        T: Into<String>,
        DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
    >(
        table_name: T,
    ) -> c3p0_json::C3p0SqliteJsonBuilder<DATA, c3p0_json::json::codec::DefaultJsonCodec> {
        c3p0_json::C3p0SqliteJsonBuilder::new(table_name)
    }

    pub fn migrate(&self) -> c3p0_migrate::C3p0MigrateBuilder<c3p0_pool_sqlite::C3p0Sqlite> {
        c3p0_migrate::C3p0MigrateBuilder::new(self.pool())
    }
}



#[derive(Clone)]
pub struct C3p0BuilderNew<C3P0: C3p0PoolManager> {
    pool_manager: C3P0,
}

impl <C3P0: C3p0PoolManager> C3p0BuilderNew<C3P0> {
    pub fn new(
        pool_manager: C3P0,
    ) -> Self {
        C3p0BuilderNew { pool_manager }
    }

    pub fn pool(&self) -> C3p0Pool<C3P0> {
        C3p0Pool::new(self.pool_manager.clone())
    }

    pub fn json<
        T: Into<String>,
        DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
    >(
        &self,
        table_name: T,
    ) -> c3p0_json::C3p0MysqlJsonBuilder<DATA, c3p0_json::json::codec::DefaultJsonCodec> {
        c3p0_json::C3p0MysqlJsonBuilder::new(table_name)
    }

    pub fn migrate(&self) -> c3p0_migrate::C3p0MigrateBuilder<c3p0_pool_mysql::C3p0Mysql> {
        c3p0_migrate::C3p0MigrateBuilder::new(self.pool())
    }
}