use crate::{C3p0Error, ForUpdate, IdType, JsonCodec, Model, NewModel};
use async_trait::async_trait;

pub mod builder;
pub mod codec;
pub mod model;

#[async_trait]
pub trait C3p0Json<Data, Codec>: Clone + Send + Sync
where
    Data: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send,
    Codec: JsonCodec<Data>,
{
    type Tx;

    fn codec(&self) -> &Codec;

    async fn create_table_if_not_exists(&self, tx: &mut Self::Tx) -> Result<(), C3p0Error>;

    async fn drop_table_if_exists(&self, tx: &mut Self::Tx, cascade: bool)
        -> Result<(), C3p0Error>;

    async fn count_all(&self, tx: &mut Self::Tx) -> Result<u64, C3p0Error>;

    async fn exists_by_id<'a, ID: Into<&'a IdType> + Send>(
        &'a self,
        tx: &mut Self::Tx,
        id: ID,
    ) -> Result<bool, C3p0Error>;

    async fn fetch_all(&self, tx: &mut Self::Tx) -> Result<Vec<Model<Data>>, C3p0Error>;

    async fn fetch_all_for_update(
        &self,
        tx: &mut Self::Tx,
        for_update: &ForUpdate,
    ) -> Result<Vec<Model<Data>>, C3p0Error>;

    async fn fetch_one_optional_by_id<'a, ID: Into<&'a IdType> + Send>(
        &'a self,
        tx: &mut Self::Tx,
        id: ID,
    ) -> Result<Option<Model<Data>>, C3p0Error>;

    async fn fetch_one_optional_by_id_for_update<'a, ID: Into<&'a IdType> + Send>(
        &'a self,
        tx: &mut Self::Tx,
        id: ID,
        for_update: &ForUpdate,
    ) -> Result<Option<Model<Data>>, C3p0Error>;

    async fn fetch_one_by_id<'a, ID: Into<&'a IdType> + Send>(
        &'a self,
        tx: &mut Self::Tx,
        id: ID,
    ) -> Result<Model<Data>, C3p0Error>;

    async fn fetch_one_by_id_for_update<'a, ID: Into<&'a IdType> + Send>(
        &'a self,
        tx: &mut Self::Tx,
        id: ID,
        for_update: &ForUpdate,
    ) -> Result<Model<Data>, C3p0Error>;

    async fn delete(&self, tx: &mut Self::Tx, obj: Model<Data>) -> Result<Model<Data>, C3p0Error>;

    async fn delete_all(&self, tx: &mut Self::Tx) -> Result<u64, C3p0Error>;

    async fn delete_by_id<'a, ID: Into<&'a IdType> + Send>(
        &'a self,
        tx: &mut Self::Tx,
        id: ID,
    ) -> Result<u64, C3p0Error>;

    async fn save(&self, tx: &mut Self::Tx, obj: NewModel<Data>) -> Result<Model<Data>, C3p0Error>;

    async fn update(&self, tx: &mut Self::Tx, obj: Model<Data>) -> Result<Model<Data>, C3p0Error>;
}

#[derive(Clone)]
pub struct Queries {
    pub id_field_name: String,
    pub version_field_name: String,
    pub create_epoch_millis_field_name: String,
    pub update_epoch_millis_field_name: String,
    pub data_field_name: String,

    pub table_name: String,
    pub schema_name: Option<String>,
    pub qualified_table_name: String,

    pub count_all_sql_query: String,
    pub exists_by_id_sql_query: String,

    pub find_base_sql_query: String,
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
