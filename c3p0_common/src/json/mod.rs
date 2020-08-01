use crate::{JsonCodec, SqlConnectionAsync, C3p0Error, IdType, Model, ForUpdate, NewModel};
use async_trait::async_trait;

pub mod builder;
pub mod codec;
pub mod model;

#[async_trait]
pub trait C3p0JsonAsync<Data, Codec>: Clone + Send + Sync
    where
        Data: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send,
        Codec: JsonCodec<Data>,
{
    type Conn: SqlConnectionAsync;

    fn codec(&self) -> &Codec;

    async fn create_table_if_not_exists(&self, conn: &mut Self::Conn) -> Result<(), C3p0Error>;

    async fn drop_table_if_exists(
        &self,
        conn: &mut Self::Conn,
        cascade: bool,
    ) -> Result<(), C3p0Error>;

    async fn count_all(&self, conn: &mut Self::Conn) -> Result<u64, C3p0Error>;

    async fn exists_by_id<'a, ID: Into<&'a IdType> + Send>(
        &'a self,
        conn: &mut Self::Conn,
        id: ID,
    ) -> Result<bool, C3p0Error>;

    async fn fetch_all(&self, conn: &mut Self::Conn) -> Result<Vec<Model<Data>>, C3p0Error>;

    async fn fetch_all_for_update(
        &self,
        conn: &mut Self::Conn,
        for_update: &ForUpdate,
    ) -> Result<Vec<Model<Data>>, C3p0Error>;

    async fn fetch_one_optional_by_id<'a, ID: Into<&'a IdType> + Send>(
        &'a self,
        conn: &mut Self::Conn,
        id: ID,
    ) -> Result<Option<Model<Data>>, C3p0Error>;

    async fn fetch_one_optional_by_id_for_update<'a, ID: Into<&'a IdType> + Send>(
        &'a self,
        conn: &mut Self::Conn,
        id: ID,
        for_update: &ForUpdate,
    ) -> Result<Option<Model<Data>>, C3p0Error>;

    async fn fetch_one_by_id<'a, ID: Into<&'a IdType> + Send>(
        &'a self,
        conn: &mut Self::Conn,
        id: ID,
    ) -> Result<Model<Data>, C3p0Error>;

    async fn fetch_one_by_id_for_update<'a, ID: Into<&'a IdType> + Send>(
        &'a self,
        conn: &mut Self::Conn,
        id: ID,
        for_update: &ForUpdate,
    ) -> Result<Model<Data>, C3p0Error>;

    async fn delete(
        &self,
        conn: &mut Self::Conn,
        obj: Model<Data>,
    ) -> Result<Model<Data>, C3p0Error>;

    async fn delete_all(&self, conn: &mut Self::Conn) -> Result<u64, C3p0Error>;

    async fn delete_by_id<'a, ID: Into<&'a IdType> + Send>(
        &'a self,
        conn: &mut Self::Conn,
        id: ID,
    ) -> Result<u64, C3p0Error>;

    async fn save(
        &self,
        conn: &mut Self::Conn,
        obj: NewModel<Data>,
    ) -> Result<Model<Data>, C3p0Error>;

    async fn update(
        &self,
        conn: &mut Self::Conn,
        obj: Model<Data>,
    ) -> Result<Model<Data>, C3p0Error>;
}


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
