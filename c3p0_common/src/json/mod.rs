use std::future::Future;

use crate::{C3p0Error, DataType, IdType, JsonCodec, Model, NewModel};

pub mod codec;
pub mod model;
pub mod types;

pub trait C3p0Json<Id, Data, Codec>: Clone + Send + Sync
where
    Id: IdType,
    Data: DataType,
    Codec: JsonCodec<Data>,
{
    type Tx<'a>;

    fn codec(&self) -> &Codec;

    fn create_table_if_not_exists(
        &self,
        tx: &mut Self::Tx<'_>,
    ) -> impl Future<Output = Result<(), C3p0Error>> + Send;

    fn drop_table_if_exists(
        &self,
        tx: &mut Self::Tx<'_>,
        cascade: bool,
    ) -> impl Future<Output = Result<(), C3p0Error>> + Send;

    fn count_all(
        &self,
        tx: &mut Self::Tx<'_>,
    ) -> impl Future<Output = Result<u64, C3p0Error>> + Send;

    fn exists_by_id(
        &self,
        tx: &mut Self::Tx<'_>,
        id: &Id,
    ) -> impl Future<Output = Result<bool, C3p0Error>> + Send;

    fn fetch_all(
        &self,
        tx: &mut Self::Tx<'_>,
    ) -> impl Future<Output = Result<Vec<Model<Id, Data>>, C3p0Error>> + Send;

    fn fetch_one_optional_by_id(
        &self,
        tx: &mut Self::Tx<'_>,
        id: &Id,
    ) -> impl Future<Output = Result<Option<Model<Id, Data>>, C3p0Error>> + Send;

    fn fetch_one_by_id(
        &self,
        tx: &mut Self::Tx<'_>,
        id: &Id,
    ) -> impl Future<Output = Result<Model<Id, Data>, C3p0Error>> + Send;

    fn delete(
        &self,
        tx: &mut Self::Tx<'_>,
        obj: Model<Id, Data>,
    ) -> impl Future<Output = Result<Model<Id, Data>, C3p0Error>> + Send;

    fn delete_all(
        &self,
        tx: &mut Self::Tx<'_>,
    ) -> impl Future<Output = Result<u64, C3p0Error>> + Send;

    fn delete_by_id(
        &self,
        tx: &mut Self::Tx<'_>,
        id: &Id,
    ) -> impl Future<Output = Result<u64, C3p0Error>> + Send;

    fn save(
        &self,
        tx: &mut Self::Tx<'_>,
        obj: NewModel<Data>,
    ) -> impl Future<Output = Result<Model<Id, Data>, C3p0Error>> + Send;

    fn update(
        &self,
        tx: &mut Self::Tx<'_>,
        obj: Model<Id, Data>,
    ) -> impl Future<Output = Result<Model<Id, Data>, C3p0Error>> + Send;
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
