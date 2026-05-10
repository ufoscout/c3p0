use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Database, query::QueryAs};

use crate::{codec::Codec, error::C3p0Error};

pub trait DataType: Sized + Send + Sync + Unpin {
    /// The name of the database table backing this type.
    ///
    /// # SQL injection
    ///
    /// This value is interpolated **verbatim** into SQL strings (via
    /// `format!`), implementors **must** therefore restrict `TABLE_NAME`
    /// to a valid SQL identifier.
    const TABLE_NAME: &'static str;
    type CODEC: Codec<Self>;
}

/// Type-level helper that lets a single generic method accept a [`DataType`] *or*
/// any wrapper around one (e.g. [`Record<T>`], [`NewRecord<T>`]) and resolve them
/// all to the same underlying `DATA` associated type.
pub trait WithData {
    /// The underlying [`DataType`] this value (or the wrapper around it) carries.
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
    pub id: i64,
    /// The version of the model used for optimistic locking.
    pub version: i64,
    /// UTC timestamp when the model was created (DB-side clock).
    pub create_time: DateTime<Utc>,
    /// UTC timestamp when the model was last updated (DB-side clock).
    pub update_time: DateTime<Utc>,
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
    /// SELECT id, version, create_time, update_time, data FROM table_name
    /// ```
    fn select_query_base() -> String {
        format!(
            "SELECT id, version, create_time, update_time, data FROM {} ",
            WITH::DATA::TABLE_NAME
        )
    }

    /// Returns a [`QueryAs`] for `Record<DATA>` whose SQL is the standard select
    /// prefix produced by [`select_query_base`](Self::select_query_base) followed by a
    /// `tail`.
    ///
    /// Concretely the produced SQL is:
    ///
    /// ```sql
    /// SELECT id, version, create_time, update_time, data FROM <TABLE_NAME> <tail>
    /// ```
    ///
    /// `tail` must therefore be a *trailing* clause that is syntactically valid after
    /// that prefix. Examples that work:
    ///
    /// - `WHERE data ->> 'username' = $1`
    /// - `ORDER BY id DESC`
    /// - `WHERE id > $1 ORDER BY id LIMIT $2`
    /// - `JOIN other_table ON ...` (the prefix already supplies `FROM <TABLE_NAME>`)
    ///
    /// Bind parameters with [`QueryAs::bind`] in the order they appear in `tail`.
    ///
    /// # SQL injection
    ///
    /// `tail` is concatenated **verbatim** into the final SQL string with no escaping
    /// or validation; it is the caller's responsibility to ensure it is trusted.
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
    ///         Record::<UserData>::query_with_tail("WHERE data ->> 'username' = $1")
    ///                 .bind(username)
    ///                 .fetch_one(conn).await
    ///      }
    /// }
    /// ```
    fn query_with_tail(
        tail: &str,
    ) -> QueryAs<'_, DB, Record<WITH::DATA>, <DB as Database>::Arguments>;

    /// Returns the number of rows in the table.
    fn count_all(tx: &mut DB::Connection) -> impl Future<Output = Result<u64, C3p0Error>>;

    /// Returns true if the entry with the given id exists.
    fn exists_by_id(
        tx: &mut DB::Connection,
        id: i64,
    ) -> impl Future<Output = Result<bool, C3p0Error>>;

    /// Returns entries in the table ordered by `id` ASC, skipping the first `offset`
    /// rows and returning at most `limit` rows. `limit = None` means no upper bound.
    fn fetch_all(
        tx: &mut DB::Connection,
        offset: u64,
        limit: Option<u64>,
    ) -> impl Future<Output = Result<Vec<Record<WITH::DATA>>, C3p0Error>>;

    /// Returns the entry with the given id. Returns None if the entry does not exist.
    fn fetch_one_optional_by_id(
        tx: &mut DB::Connection,
        id: i64,
    ) -> impl Future<Output = Result<Option<Record<WITH::DATA>>, C3p0Error>>;

    /// Returns the entry with the given id. Returns an error if the entry does not exist.
    fn fetch_one_by_id(
        tx: &mut DB::Connection,
        id: i64,
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
        id: i64,
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
