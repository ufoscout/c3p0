use crate::error::C3p0Error;
use crate::json::codec::JsonCodec;
use crate::json::model::*;
use crate::sql::ForUpdate;

pub mod builder;
pub mod codec;
pub mod model;

#[derive(Clone)]
pub struct Queries {
    pub id_field_name: String,
    pub version_field_name: String,
    pub data_field_name: String,

    pub table_name: String,
    pub schema_name: Option<String>,
    pub qualified_table_name: String,

    pub count_all_sql_query: String,
    pub exists_by_id_sql_query: String,

    pub find_all_sql_query: String,
    pub find_by_id_sql_query: String,

    pub delete_sql_query: String,
    pub delete_all_sql_query: String,
    pub delete_by_id_sql_query: String,

    pub save_sql_query: String,

    pub update_sql_query: String,

    pub create_table_sql_query: String,
    pub drop_table_sql_query: String,
    pub drop_table_sql_query_cascade: String,
    pub lock_table_sql_query: Option<String>,
}

pub trait C3p0Json<DATA, CODEC>: Clone
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
    CODEC: JsonCodec<DATA>,
{
    type CONN;

    fn codec(&self) -> &CODEC;

    fn create_table_if_not_exists(&self, conn: &mut Self::CONN) -> Result<(), C3p0Error>;

    fn drop_table_if_exists(&self, conn: &mut Self::CONN, cascade: bool) -> Result<(), C3p0Error>;

    fn count_all(&self, conn: &mut Self::CONN) -> Result<u64, C3p0Error>;

    fn exists_by_id<'a, ID: Into<&'a IdType>>(
        &'a self,
        conn: &mut Self::CONN,
        id: ID,
    ) -> Result<bool, C3p0Error>;

    fn fetch_all(&self, conn: &mut Self::CONN) -> Result<Vec<Model<DATA>>, C3p0Error>;

    fn fetch_all_for_update(
        &self,
        conn: &mut Self::CONN,
        for_update: &ForUpdate,
    ) -> Result<Vec<Model<DATA>>, C3p0Error>;

    fn fetch_one_optional_by_id<'a, ID: Into<&'a IdType>>(
        &'a self,
        conn: &mut Self::CONN,
        id: ID,
    ) -> Result<Option<Model<DATA>>, C3p0Error>;

    fn fetch_one_optional_by_id_for_update<'a, ID: Into<&'a IdType>>(
        &'a self,
        conn: &mut Self::CONN,
        id: ID,
        for_update: &ForUpdate,
    ) -> Result<Option<Model<DATA>>, C3p0Error>;

    fn fetch_one_by_id<'a, ID: Into<&'a IdType>>(
        &'a self,
        conn: &mut Self::CONN,
        id: ID,
    ) -> Result<Model<DATA>, C3p0Error> {
        self.fetch_one_optional_by_id(conn, id)
            .and_then(|result| result.ok_or_else(|| C3p0Error::ResultNotFoundError))
    }

    fn fetch_one_by_id_for_update<'a, ID: Into<&'a IdType>>(
        &'a self,
        conn: &mut Self::CONN,
        id: ID,
        for_update: &ForUpdate,
    ) -> Result<Model<DATA>, C3p0Error> {
        self.fetch_one_optional_by_id_for_update(conn, id, for_update)
            .and_then(|result| result.ok_or_else(|| C3p0Error::ResultNotFoundError))
    }

    fn delete(&self, conn: &mut Self::CONN, obj: Model<DATA>) -> Result<Model<DATA>, C3p0Error>;

    fn delete_all(&self, conn: &mut Self::CONN) -> Result<u64, C3p0Error>;

    fn delete_by_id<'a, ID: Into<&'a IdType>>(
        &'a self,
        conn: &mut Self::CONN,
        id: ID,
    ) -> Result<u64, C3p0Error>;

    fn save(&self, conn: &mut Self::CONN, obj: NewModel<DATA>) -> Result<Model<DATA>, C3p0Error>;

    fn update(&self, conn: &mut Self::CONN, obj: Model<DATA>) -> Result<Model<DATA>, C3p0Error>;
}

#[cfg(feature = "async")]
use async_trait::async_trait;

#[cfg(feature = "async")]
#[async_trait]
pub trait C3p0JsonAsync<DATA, CODEC>: Clone
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
    CODEC: JsonCodec<DATA>,
{
    type CONN;

    fn codec(&self) -> &CODEC;

    async fn create_table_if_not_exists(&self, conn: &Self::CONN) -> Result<(), C3p0Error>;

    async fn drop_table_if_exists(&self, conn: &Self::CONN, cascade: bool)
        -> Result<(), C3p0Error>;

    async fn count_all(&self, conn: &Self::CONN) -> Result<u64, C3p0Error>;

    async fn exists_by_id<'a, ID: Into<&'a IdType>>(
        &'a self,
        conn: &Self::CONN,
        id: ID,
    ) -> Result<bool, C3p0Error>;

    async fn fetch_all(&self, conn: &Self::CONN) -> Result<Vec<Model<DATA>>, C3p0Error>;

    async fn fetch_all_for_update(
        &self,
        conn: &Self::CONN,
        for_update: &ForUpdate,
    ) -> Result<Vec<Model<DATA>>, C3p0Error>;

    async fn fetch_one_optional_by_id<'a, ID: Into<&'a IdType>>(
        &'a self,
        conn: &Self::CONN,
        id: ID,
    ) -> Result<Option<Model<DATA>>, C3p0Error>;

    async fn fetch_one_optional_by_id_for_update<'a, ID: Into<&'a IdType>>(
        &'a self,
        conn: &Self::CONN,
        id: ID,
        for_update: &ForUpdate,
    ) -> Result<Option<Model<DATA>>, C3p0Error>;

    async fn fetch_one_by_id<'a, ID: Into<&'a IdType>>(
        &'a self,
        conn: &Self::CONN,
        id: ID,
    ) -> Result<Model<DATA>, C3p0Error>;

    async fn fetch_one_by_id_for_update<'a, ID: Into<&'a IdType>>(
        &'a self,
        conn: &Self::CONN,
        id: ID,
        for_update: &ForUpdate,
    ) -> Result<Model<DATA>, C3p0Error>;

    async fn delete(&self, conn: &Self::CONN, obj: &Model<DATA>) -> Result<u64, C3p0Error>;

    async fn delete_all(&self, conn: &Self::CONN) -> Result<u64, C3p0Error>;

    async fn delete_by_id<'a, ID: Into<&'a IdType>>(
        &'a self,
        conn: &Self::CONN,
        id: ID,
    ) -> Result<u64, C3p0Error>;

    async fn save(&self, conn: &Self::CONN, obj: NewModel<DATA>) -> Result<Model<DATA>, C3p0Error>;

    async fn update(&self, conn: &Self::CONN, obj: Model<DATA>) -> Result<Model<DATA>, C3p0Error>;
}
