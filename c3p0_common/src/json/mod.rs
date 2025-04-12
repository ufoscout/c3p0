use std::future::Future;

use crate::{C3p0Error, DataType, IdType, JsonCodec, Model, NewModel};

pub mod codec;
pub mod model;
pub mod types;

/// Defines the default API to interact with a database table.
pub trait C3p0Json<Id, Data, Codec>: Clone + Send + Sync
where
    Id: IdType,
    Data: DataType,
    Codec: JsonCodec<Data>,
{
    type Tx<'a>;

    /// Returns the JSON codec.
    fn codec(&self) -> &Codec;

    /// Creates the table if it does not exist.
    fn create_table_if_not_exists(
        &self,
        tx: &mut Self::Tx<'_>,
    ) -> impl Future<Output = Result<(), C3p0Error>> + Send;

    /// Drops the table if it exists.
    fn drop_table_if_exists(
        &self,
        tx: &mut Self::Tx<'_>,
        cascade: bool,
    ) -> impl Future<Output = Result<(), C3p0Error>> + Send;

    /// Returns the number of rows in the table.
    fn count_all(
        &self,
        tx: &mut Self::Tx<'_>,
    ) -> impl Future<Output = Result<u64, C3p0Error>> + Send;

    /// Returns true if the entry with the given id exists.
    fn exists_by_id(
        &self,
        tx: &mut Self::Tx<'_>,
        id: &Id,
    ) -> impl Future<Output = Result<bool, C3p0Error>> + Send;

    /// Returns all entries in the table.
    fn fetch_all(
        &self,
        tx: &mut Self::Tx<'_>,
    ) -> impl Future<Output = Result<Vec<Model<Id, Data>>, C3p0Error>> + Send;

    /// Returns the entry with the given id. Returns None if the entry does not exist.
    fn fetch_one_optional_by_id(
        &self,
        tx: &mut Self::Tx<'_>,
        id: &Id,
    ) -> impl Future<Output = Result<Option<Model<Id, Data>>, C3p0Error>> + Send;

    /// Returns the entry with the given id. Returns an error if the entry does not exist.
    fn fetch_one_by_id(
        &self,
        tx: &mut Self::Tx<'_>,
        id: &Id,
    ) -> impl Future<Output = Result<Model<Id, Data>, C3p0Error>> + Send;

    /// Deletes the entry with the given id.
    fn delete(
        &self,
        tx: &mut Self::Tx<'_>,
        obj: Model<Id, Data>,
    ) -> impl Future<Output = Result<Model<Id, Data>, C3p0Error>> + Send;

    /// Deletes all entries in the table.
    fn delete_all(
        &self,
        tx: &mut Self::Tx<'_>,
    ) -> impl Future<Output = Result<u64, C3p0Error>> + Send;

    /// Deletes the entry with the given id.
    fn delete_by_id(
        &self,
        tx: &mut Self::Tx<'_>,
        id: &Id,
    ) -> impl Future<Output = Result<u64, C3p0Error>> + Send;

    /// Creates a new entry.
    fn save(
        &self,
        tx: &mut Self::Tx<'_>,
        obj: NewModel<Data>,
    ) -> impl Future<Output = Result<Model<Id, Data>, C3p0Error>> + Send;

    /// Updates the entry with the given id. Returns an error if the entry does not exist.
    /// This uses optimistic locking by using the version field to detect update conflicts; it will update the entry and will throw an error if the version does not match.
    /// The version field is incremented by 1 for each update.
    fn update(
        &self,
        tx: &mut Self::Tx<'_>,
        obj: Model<Id, Data>,
    ) -> impl Future<Output = Result<Model<Id, Data>, C3p0Error>> + Send;
}

/// Contains all the SQL queries for a JSON table.
/// This is used to generate the SQL queries for the table for a specific database.
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
