use crate::{C3p0Error, DataType, IdType, JsonCodec, Model, NewModel};
use async_trait::async_trait;

pub mod codec;
pub mod model;
pub mod types;

#[async_trait]
pub trait C3p0Json<Id, Data, Codec>: Clone + Send + Sync
where
    Id: IdType,
    Data: DataType,
    Codec: JsonCodec<Data>,
{
    type Tx;

    fn codec(&self) -> &Codec;

    async fn create_table_if_not_exists(&self, tx: &mut Self::Tx) -> Result<(), C3p0Error>;

    async fn drop_table_if_exists(&self, tx: &mut Self::Tx, cascade: bool)
        -> Result<(), C3p0Error>;

    async fn count_all(&self, tx: &mut Self::Tx) -> Result<u64, C3p0Error>;

    async fn exists_by_id(&self, tx: &mut Self::Tx, id: &Id) -> Result<bool, C3p0Error>;

    async fn fetch_all(&self, tx: &mut Self::Tx) -> Result<Vec<Model<Id, Data>>, C3p0Error>;

    async fn fetch_one_optional_by_id(
        &self,
        tx: &mut Self::Tx,
        id: &Id,
    ) -> Result<Option<Model<Id, Data>>, C3p0Error>;

    async fn fetch_one_by_id(
        &self,
        tx: &mut Self::Tx,
        id: &Id,
    ) -> Result<Model<Id, Data>, C3p0Error>;

    async fn delete(
        &self,
        tx: &mut Self::Tx,
        obj: Model<Id, Data>,
    ) -> Result<Model<Id, Data>, C3p0Error>;

    async fn delete_all(&self, tx: &mut Self::Tx) -> Result<u64, C3p0Error>;

    async fn delete_by_id(&self, tx: &mut Self::Tx, id: &Id) -> Result<u64, C3p0Error>;

    async fn save(
        &self,
        tx: &mut Self::Tx,
        obj: NewModel<Data>,
    ) -> Result<Model<Id, Data>, C3p0Error>;

    async fn update(
        &self,
        tx: &mut Self::Tx,
        obj: Model<Id, Data>,
    ) -> Result<Model<Id, Data>, C3p0Error>;
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
    pub save_sql_query_with_id: String,

    pub update_sql_query: String,

    pub create_table_sql_query: String,
    pub drop_table_sql_query: String,
    pub drop_table_sql_query_cascade: String,
    pub lock_table_sql_query: Option<String>,
}
