use serde::{Deserialize, Serialize};
use sqlx::{ColumnIndex, Database, Decode, FromRow, Row, Type, query::QueryAs};

use crate::{codec::Codec, error::C3p0Error};

pub trait DataType: Sized + Send + Sync + Unpin {
    const TABLE_NAME: &'static str;
    type CODEC: Codec<Self>;
}

pub trait WithData {
    type DATA: DataType;
}

impl<DATA: DataType> WithData for DATA {
    type DATA = DATA;
}

impl<DATA: DataType> WithData for Record<DATA> {
    type DATA = DATA;
}

impl<DATA: DataType> WithData for NewRecord<DATA> {
    type DATA = DATA;
}

/// A model for a database table.
/// This is used to retrieve and update an entry in a database table.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Record<DATA: DataType> {
    /// The unique identifier of the model.
    pub id: u64,
    /// The version of the model used for optimistic locking.
    pub version: u32,
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

pub trait DbOps<DB: Database, WITH: WithData> {
    /// Returns a SQL query string to select all columns from the database table. I.e.:
    ///
    /// ```sql
    /// SELECT id, version, create_epoch_millis, update_epoch_millis, data FROM table_name
    /// ```
    fn select_query_base() -> String {
        format!(
            "SELECT id, version, create_epoch_millis, update_epoch_millis, data FROM {}",
            WITH::DATA::TABLE_NAME
        )
    }

    /// Returns a QueryAs object that can be used to query the database table.
    /// The query string should be a valid SQL query that can be appended to the
    /// select query base string.
    ///
    /// # Examples
    ///
    /// ```rust
    /// #[cfg(feature = "postgres")]
    /// pub mod with_postgres {
    ///
    ///     use c3p0::{DataType, DbOps, Record};
    ///
    ///     /// Example of a model for a database table
    ///     #[derive(Clone, serde::Serialize, serde::Deserialize, Debug, PartialEq)]
    ///     pub struct UserData {
    ///         pub username: String,
    ///     }
    ///
    ///     /// Implement the Data trait for the UserData model using the table "USER_DATA"
    ///     impl DataType for UserData {
    ///         const TABLE_NAME: &'static str = "USER_DATA";
    ///         type CODEC = Self;
    ///     }
    ///
    ///     pub async fn find_by_username(conn: &mut sqlx::PgConnection, username: &str) -> Result<Record<UserData>, sqlx::Error> {
    ///         Record::<UserData>::query_with("where data ->> 'username' = $1")
    ///                 .bind(username)
    ///                 .fetch_one(conn).await
    ///      }
    /// }
    /// ```
    fn query_with(sql: &str) -> QueryAs<'_, DB, Record<WITH::DATA>, <DB as Database>::Arguments>;

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
    ) -> impl Future<Output = Result<Vec<Record<WITH::DATA>>, C3p0Error>>;

    /// Returns the entry with the given id. Returns None if the entry does not exist.
    fn fetch_one_optional_by_id(
        tx: &mut DB::Connection,
        id: u64,
    ) -> impl Future<Output = Result<Option<Record<WITH::DATA>>, C3p0Error>>;

    /// Returns the entry with the given id. Returns an error if the entry does not exist.
    fn fetch_one_by_id(
        tx: &mut DB::Connection,
        id: u64,
    ) -> impl Future<Output = Result<Record<WITH::DATA>, C3p0Error>>;

    /// Deletes the entry with the given id.
    fn delete(
        self,
        tx: &mut DB::Connection,
    ) -> impl Future<Output = Result<Record<WITH::DATA>, C3p0Error>>;

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
    ) -> impl Future<Output = Result<Record<WITH::DATA>, C3p0Error>>;
}

pub trait DbSave<DB: Database, WITH: WithData> {
    /// Creates a new entry.
    fn save(
        self,
        tx: &mut DB::Connection,
    ) -> impl Future<Output = Result<Record<WITH::DATA>, C3p0Error>>;
}

impl<DATA: DataType, DB: Database, R: Row<Database = DB>> FromRow<'_, R> for Record<DATA>
where
    usize: ColumnIndex<R>,
    for<'c> i32: Type<DB> + Decode<'c, DB>,
    for<'c> i64: Type<DB> + Decode<'c, DB>,
    for<'c> serde_json::value::Value: Type<DB> + Decode<'c, DB>,
{
    fn from_row(row: &R) -> Result<Self, sqlx::Error> {
        row_to_record_with_index(row, 0, 1, 2, 3, 4)
    }
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
) -> Result<Record<DATA>, sqlx::Error>
where
    for<'c> i32: Type<DB> + Decode<'c, DB>,
    for<'c> i64: Type<DB> + Decode<'c, DB>,
    for<'c> serde_json::value::Value: Type<DB> + Decode<'c, DB>,
{
    let id: i64 = row.try_get(id_index)?;

    let version: i32 = row.try_get(version_index)?;

    let create_epoch_millis: i64 = row.try_get(create_epoch_millis_index)?;

    let update_epoch_millis: i64 = row.try_get(update_epoch_millis_index)?;

    let data: DATA::CODEC = serde_json::from_value(row.try_get(data_index)?)
        .map_err(|e| sqlx::Error::Decode(Box::new(e)))?;

    Ok(Record {
        id: id as u64,
        version: version as u32,
        data: DATA::CODEC::decode(data),
        create_epoch_millis,
        update_epoch_millis,
    })
}
