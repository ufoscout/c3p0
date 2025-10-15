use sqlx::Database;

use crate::{C3p0Error, DataType, NewRecord, Record, WithData};

/// A trait for a transaction.
pub trait Tx {
    type DB: Database;

    /// Creates the table if it does not exist.
    fn create_table_if_not_exists<DATA: WithData>(
        &mut self,
    ) -> impl Future<Output = Result<(), C3p0Error>>;

    /// Drops the table if it exists.
    fn drop_table_if_exists<DATA: WithData>(
        &mut self,
        cascade: bool,
    ) -> impl Future<Output = Result<(), C3p0Error>>;

    /// Returns the number of rows in the table.
    fn count_all<DATA: WithData>(&mut self) -> impl Future<Output = Result<u64, C3p0Error>>;

    /// Returns true if the entry with the given id exists.
    fn exists_by_id<DATA: WithData>(
        &mut self,
        id: u64,
    ) -> impl Future<Output = Result<bool, C3p0Error>>;

    /// Returns all entries in the table.
    fn fetch_all<DATA: DataType>(
        &mut self,
    ) -> impl Future<Output = Result<Vec<Record<DATA>>, C3p0Error>>;

    /// Returns the entry with the given id. Returns None if the entry does not exist.
    fn fetch_one_optional_by_id<DATA: DataType>(
        &mut self,
        id: u64,
    ) -> impl Future<Output = Result<Option<Record<DATA>>, C3p0Error>>;

    /// Returns the entry with the given id. Returns an error if the entry does not exist.
    fn fetch_one_by_id<DATA: DataType>(
        &mut self,
        id: u64,
    ) -> impl Future<Output = Result<Record<DATA>, C3p0Error>>;

    /// Deletes the entry with the given id.
    fn delete<DATA: DataType>(
        &mut self,
        record: Record<DATA>,
    ) -> impl Future<Output = Result<Record<DATA>, C3p0Error>>;

    /// Deletes all entries in the table.
    fn delete_all<DATA: WithData>(&mut self) -> impl Future<Output = Result<u64, C3p0Error>>;

    /// Deletes the entry with the given id.
    fn delete_by_id<DATA: WithData>(
        &mut self,
        id: u64,
    ) -> impl Future<Output = Result<u64, C3p0Error>>;

    /// Updates the entry with the given id. Returns an error if the entry does not exist.
    /// This uses optimistic locking by using the version field to detect update conflicts; it will update the entry and will throw an error if the version does not match.
    /// The version field is incremented by 1 for each update.
    fn update<DATA: DataType>(
        &mut self,
        record: Record<DATA>,
    ) -> impl Future<Output = Result<Record<DATA>, C3p0Error>>;

    /// Creates a new entry.
    fn save<DATA: DataType>(
        &mut self,
        record: NewRecord<DATA>,
    ) -> impl Future<Output = Result<Record<DATA>, C3p0Error>>;
}
