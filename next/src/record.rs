use serde::{Deserialize, Serialize};
use sqlx::{query::Query, ColumnIndex, Database, Decode, IntoArguments, Row, Type};

use crate::{codec::Codec, error::C3p0Error};

pub trait Data: Sized + Send + Sync {
    const TABLE_NAME: &'static str;
    type CODEC: Codec<Self>;
}

/// A model for a database table.
/// This is used to retrieve and update an entry in a database table.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Record<DATA: Data> {
    /// The unique identifier of the model.
    pub id: u64,
    /// The version of the model used for optimistic locking.
    pub version: i32,
    /// The epoch millis when the model was created.
    #[serde(default)]
    pub create_epoch_millis: i64,
    /// The epoch millis when the model was last updated.
    #[serde(default)]
    pub update_epoch_millis: i64,
    /// The data of the model.
    pub data: DATA,
}


/// A new model for a database table.
/// This is used to create a new entry in a database table.
pub struct NewRecord<DATA> {
    pub data: DATA
}

impl<DATA: Data> NewRecord<DATA> {
    /// Creates a new `NewRecord` instance from a `Data` value.
    /// Sets the version to 0.
    pub fn new(data: DATA) -> Self {
        NewRecord { data }
    }

}

impl<DATA: Data + Default> Default for NewRecord<DATA> {
    fn default() -> Self {
        NewRecord::new(DATA::default())
    }
}

impl<DATA> From<DATA> for NewRecord<DATA>
where
    DATA: Data,
{
    fn from(data: DATA) -> Self {
        NewRecord::new(data)
    }
}

impl<DATA: Data> Record<DATA> {


    /// Returns a SQL query string to select all columns from the database table.
    pub(crate) fn select_query_base() -> String {
        format!(
            "SELECT id, version, create_epoch_millis, update_epoch_millis, data FROM {}",
            DATA::TABLE_NAME
        )
    }
    
    /// Converts the current `Record` instance into a `NewRecord` instance,
    /// resetting the version to the initial state while retaining the data.
    pub(crate) fn into_new(self) -> NewRecord<DATA> {
        NewRecord::new(self.data)
    }

    /// Creates a new `Record` instance from a `NewRecord` instance.
    ///
    /// - `id`: The unique identifier of the model.
    /// - `create_epoch_millis`: The epoch millis when the model was created.
    /// - `model`: The `NewRecord` instance to create the `Record` instance from.
    ///
    /// Returns a `Record` instance with the version set to the initial state,
    /// the create and update epoch millis set to the given `create_epoch_millis`,
    /// and the data set to the data of the `model` parameter.
    pub(crate) fn from_new(
        id: u64,
        create_epoch_millis: i64,
        model: NewRecord<DATA>,
    ) -> Record<DATA> {
        Record {
            id,
            version: 0,
            create_epoch_millis,
            update_epoch_millis: create_epoch_millis,
            data: model.data,
        }
    }

    /// Creates a new `Record` instance from the current `Record` instance,
    /// incrementing the version by one and updating the update epoch millis
    /// to the given `update_epoch_millis`.
    ///
    /// - `update_epoch_millis`: The epoch millis when the model was last updated.
    ///
    /// Returns a `Record` instance with the version incremented by one,
    /// the create epoch millis unchanged, the update epoch millis set to
    /// the given `update_epoch_millis`, and the data unchanged.
    pub(crate) fn into_new_version(self, update_epoch_millis: i64) -> Record<DATA> {
        Record {
            id: self.id,
            version: self.version + 1,
            create_epoch_millis: self.create_epoch_millis,
            update_epoch_millis,
            data: self.data,
        }
    }
}

pub trait DbRead<DB: Database, DATA: Data> {

    /// Allows the execution of a custom sql query and returns all the entries in the result set.
    /// For this to work, the sql query:
    /// - must be a SELECT
    /// - must declare the ID, VERSION and Data fields in this exact order
    fn fetch_all_with_sql<'a, A: 'a + Send + IntoArguments<'a, DB>>(
        tx: &mut DB::Connection,
        sql: Query<'a, DB, A>,
    ) -> impl Future<Output = Result<Vec<Record<DATA>>, C3p0Error>>;

    /// Allows the execution of a custom sql query and returns the first entry in the result set.
    /// For this to work, the sql query:
    /// - must be a SELECT
    /// - must declare the ID, VERSION and Data fields in this exact order
    fn fetch_one_optional_with_sql<'a, A: 'a + Send + IntoArguments<'a, DB>>(
        tx: &mut DB::Connection,
        sql: Query<'a, DB, A>,
    ) -> impl Future<Output = Result<Option<Record<DATA>>, C3p0Error>>;

    /// Allows the execution of a custom sql query and returns the first entry in the result set.
    /// For this to work, the sql query:
    /// - must be a SELECT
    /// - must declare the ID, VERSION and Data fields in this exact order
    fn fetch_one_with_sql<'a, A: 'a + Send + IntoArguments<'a, DB>>(
        tx: &mut DB::Connection,
        sql: Query<'a, DB, A>,
    ) ->  impl Future<Output = Result<Record<DATA>, C3p0Error>>;

        fn count_all(tx: &mut DB::Connection) -> impl Future<Output = Result<u64, C3p0Error>>;

    fn exists_by_id(tx: &mut DB::Connection, id: u64) -> impl Future<Output = Result<bool, C3p0Error>>;

    fn fetch_all(tx: &mut DB::Connection) -> impl Future<Output = Result<Vec<Record<DATA>>, C3p0Error>>;

    fn fetch_one_optional_by_id(
        tx: &mut DB::Connection,
        id: u64,
    ) -> impl Future<Output = Result<Option<Record<DATA>>, C3p0Error>>;

    fn fetch_one_by_id(tx: &mut DB::Connection, id: u64) -> impl Future<Output = Result<Record<DATA>, C3p0Error>> + Send;

    fn delete(
        self,
        tx: &mut DB::Connection,
    ) -> impl Future<Output = Result<Record<DATA>, C3p0Error>>;

    fn delete_all(tx: &mut DB::Connection) -> impl Future<Output = Result<u64, C3p0Error>>;

    fn delete_by_id(tx: &mut DB::Connection, id: u64) -> impl Future<Output = Result<u64, C3p0Error>>;

}

pub trait DbWrite<DB: Database, DATA: Data> {
    fn save(self, tx: &mut DB::Connection) -> impl Future<Output = Result<Record<DATA>, C3p0Error>> + Send;
}

/// Converts a row to a Model
#[allow(clippy::too_many_arguments)]
#[inline]
pub fn row_to_record_with_index<
    DATA: Data,
    R: Row<Database = DB>,
    IdIdx: ColumnIndex<R>,
    VersionIdx: ColumnIndex<R>,
    CreateEpochMillisIdx: ColumnIndex<R>,
    UpdateEpochMillisIdx: ColumnIndex<R>,
    DataIdx: ColumnIndex<R>,
    DB: Database,
>(
    row: &R,
    id_index: IdIdx,
    version_index: VersionIdx,
    create_epoch_millis_index: CreateEpochMillisIdx,
    update_epoch_millis_index: UpdateEpochMillisIdx,
    data_index: DataIdx,
) -> Result<Record<DATA>, C3p0Error>
where
    for<'c> i32: Type<DB> + Decode<'c, DB>,
    for<'c> i64: Type<DB> + Decode<'c, DB>,
    for<'c> serde_json::value::Value: Type<DB> + Decode<'c, DB>,
{
    let id: i64 = row.try_get(id_index).map_err(|err| C3p0Error::RowMapperError {
            cause: format!("Row contains no values for id index. Err: {err:?}"),
        })?;

    let version: i32 =
        row.try_get(version_index)
            .map_err(|err| C3p0Error::RowMapperError {
                cause: format!("Row contains no values for version index. Err: {err:?}"),
            })?;
    
    let create_epoch_millis: i64 =
        row.try_get(create_epoch_millis_index)
            .map_err(|err| C3p0Error::RowMapperError {
                cause: format!(
                    "Row contains no values for create_epoch_millis index. Err: {err:?}"
                ),
            })?;
    let update_epoch_millis: i64 =
        row.try_get(update_epoch_millis_index)
            .map_err(|err| C3p0Error::RowMapperError {
                cause: format!(
                    "Row contains no values for update_epoch_millis index. Err: {err:?}"
                ),
            })?;

    let data: DATA::CODEC = serde_json::from_value(row.try_get(data_index).map_err(|err| {
        C3p0Error::RowMapperError {
            cause: format!("Row contains no values for data index. Err: {err:?}"),
        }
    })?)?;

    Ok(Record {
        id: id as u64,
        version,
        data: DATA::CODEC::decode(data),
        create_epoch_millis,
        update_epoch_millis,
    })
}
