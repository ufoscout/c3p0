use serde::{Deserialize, Serialize};
use sqlx::{ColumnIndex, Database, Decode, IntoArguments, Row, Type, query::Query};

use crate::{codec::Codec, error::C3p0Error};

pub trait DataType: Sized + Send + Sync {
    const TABLE_NAME: &'static str;
    type CODEC: Codec<Self>;
}

/// A model for a database table.
/// This is used to retrieve and update an entry in a database table.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Record<DATA: DataType> {
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
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct NewRecord<DATA> {
    pub data: DATA,
}

impl<DATA: DataType> NewRecord<DATA> {
    /// Creates a new `NewRecord` instance from a `Data` value.
    /// Sets the version to 0.
    pub fn new(data: DATA) -> Self {
        NewRecord { data }
    }
}

impl<DATA: DataType + Default> Default for NewRecord<DATA> {
    fn default() -> Self {
        NewRecord::new(DATA::default())
    }
}

impl<DATA> From<DATA> for NewRecord<DATA>
where
    DATA: DataType,
{
    fn from(data: DATA) -> Self {
        NewRecord::new(data)
    }
}

pub trait DbRead<DB: Database, DATA: DataType> {
    
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
    ) -> impl Future<Output = Result<Record<DATA>, C3p0Error>>;

    /// Returns the number of rows in the table.
    fn count_all(tx: &mut DB::Connection) -> impl Future<Output = Result<u64, C3p0Error>>;

    /// Returns true if the entry with the given id exists.
    fn exists_by_id(
        tx: &mut DB::Connection,
        id: u64,
    ) -> impl Future<Output = Result<bool, C3p0Error>>;

    /// Returns all the entries in the table.
    fn fetch_all(
        tx: &mut DB::Connection,
    ) -> impl Future<Output = Result<Vec<Record<DATA>>, C3p0Error>>;

    /// Returns the entry with the given id. Returns None if the entry does not exist.
    fn fetch_one_optional_by_id(
        tx: &mut DB::Connection,
        id: u64,
    ) -> impl Future<Output = Result<Option<Record<DATA>>, C3p0Error>>;

    /// Returns the entry with the given id. Returns an error if the entry does not exist.
    fn fetch_one_by_id(
        tx: &mut DB::Connection,
        id: u64,
    ) -> impl Future<Output = Result<Record<DATA>, C3p0Error>> + Send;

    /// Deletes the entry with the given id.
    fn delete(
        self,
        tx: &mut DB::Connection,
    ) -> impl Future<Output = Result<Record<DATA>, C3p0Error>>;

    /// Deletes all entries in the table.
    fn delete_all(tx: &mut DB::Connection) -> impl Future<Output = Result<u64, C3p0Error>>;

    /// Deletes the entry with the given id.
    fn delete_by_id(
        tx: &mut DB::Connection,
        id: u64,
    ) -> impl Future<Output = Result<u64, C3p0Error>>;

    /// Updates the entry with the given id. Returns an error if the entry does not exist.
    /// This uses optimistic locking by using the version field to detect update conflicts; it will update the entry and will throw an error if the version does not match.
    /// The version field is incremented by 1 for each update.
    fn update(
        self,
        tx: &mut DB::Connection,
    ) -> impl Future<Output = Result<Record<DATA>, C3p0Error>>;
}

pub trait DbWrite<DB: Database, DATA: DataType> {
    /// Creates a new entry.
    fn save(
        self,
        tx: &mut DB::Connection,
    ) -> impl Future<Output = Result<Record<DATA>, C3p0Error>> + Send;
}

/// Converts a row to a Model
#[allow(clippy::too_many_arguments)]
#[inline]
pub fn row_to_record_with_index<
    DATA: DataType,
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
    let id: i64 = row
        .try_get(id_index)
        .map_err(|err| C3p0Error::RowMapperError {
            cause: format!("Row contains no values for id index. Err: {err:?}"),
        })?;

    let version: i32 = row
        .try_get(version_index)
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
