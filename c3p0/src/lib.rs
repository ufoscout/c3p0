pub use c3p0_json::*;

pub mod migrate {
    pub use c3p0_migrate::migration::*;
    pub use c3p0_migrate::*;
}

pub struct C3p0Builder;

#[cfg(feature = "mysql")]
impl C3p0Builder {
    pub fn pool() -> c3p0_pool_mysql::C3p0MysqlBuilder {
        c3p0_pool_mysql::C3p0MysqlBuilder {}
    }

    pub fn json<
        T: Into<String>,
        DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
    >(
        table_name: T,
    ) -> c3p0_json::C3p0MysqlJsonBuilder<DATA, c3p0_json::json::codec::DefaultJsonCodec> {
        c3p0_json::C3p0MysqlJsonBuilder::new(table_name)
    }

    pub fn migrate(
        c3p0: c3p0_pool_mysql::C3p0Mysql,
    ) -> c3p0_migrate::C3p0MigrateBuilder<c3p0_pool_mysql::C3p0Mysql> {
        c3p0_migrate::C3p0MigrateBuilder::new(c3p0)
    }
}

#[cfg(feature = "pg")]
impl C3p0Builder {
    pub fn pool() -> c3p0_pool_pg::C3p0PgBuilder {
        c3p0_pool_pg::C3p0PgBuilder {}
    }

    pub fn json<
        T: Into<String>,
        DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
    >(
        table_name: T,
    ) -> c3p0_json::C3p0PgJsonBuilder<DATA, c3p0_json::json::codec::DefaultJsonCodec> {
        c3p0_json::C3p0PgJsonBuilder::new(table_name)
    }

    pub fn migrate(
        c3p0: c3p0_pool_pg::C3p0Pg,
    ) -> c3p0_migrate::C3p0MigrateBuilder<c3p0_pool_pg::C3p0Pg> {
        c3p0_migrate::C3p0MigrateBuilder::new(c3p0)
    }
}

#[cfg(feature = "sqlite")]
impl C3p0Builder {
    pub fn pool() -> c3p0_pool_sqlite::C3p0SqliteBuilder {
        c3p0_pool_sqlite::C3p0SqliteBuilder {}
    }

    pub fn json<
        T: Into<String>,
        DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
    >(
        table_name: T,
    ) -> c3p0_json::C3p0SqliteJsonBuilder<DATA, c3p0_json::json::codec::DefaultJsonCodec> {
        c3p0_json::C3p0SqliteJsonBuilder::new(table_name)
    }

    pub fn migrate(
        c3p0: c3p0_pool_sqlite::C3p0Sqlite,
    ) -> c3p0_migrate::C3p0MigrateBuilder<c3p0_pool_sqlite::C3p0Sqlite> {
        c3p0_migrate::C3p0MigrateBuilder::new(c3p0)
    }
}
