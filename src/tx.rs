use sqlx::{query::Query, Database, IntoArguments};

use crate::{C3p0Error, Data, NewRecord, Record};


pub trait Tx<DB: Database> {

    fn create_table_if_not_exists<DATA: Data>(&mut self) -> impl Future<Output = Result<(), C3p0Error>>;

    fn drop_table_if_exists<DATA: Data>(
        &mut self,
        cascade: bool,
    ) -> impl Future<Output = Result<(), C3p0Error>>;

    /// Allows the execution of a custom sql query and returns all the entries in the result set.
    /// For this to work, the sql query:
    /// - must be a SELECT
    /// - must declare the ID, VERSION and Data fields in this exact order
    fn fetch_all_with_sql<'a, DATA: Data, A: 'a + Send + IntoArguments<'a, DB>>(
        &mut self,
        sql: Query<'a, DB, A>,
    ) -> impl Future<Output = Result<Vec<Record<DATA>>, C3p0Error>>;

    /// Allows the execution of a custom sql query and returns the first entry in the result set.
    /// For this to work, the sql query:
    /// - must be a SELECT
    /// - must declare the ID, VERSION and Data fields in this exact order
    fn fetch_one_optional_with_sql<'a, DATA: Data, A: 'a + Send + IntoArguments<'a, DB>>(
        &mut self,
        sql: Query<'a, DB, A>,
    ) -> impl Future<Output = Result<Option<Record<DATA>>, C3p0Error>>;

    /// Allows the execution of a custom sql query and returns the first entry in the result set.
    /// For this to work, the sql query:
    /// - must be a SELECT
    /// - must declare the ID, VERSION and Data fields in this exact order
    fn fetch_one_with_sql<'a, DATA: Data, A: 'a + Send + IntoArguments<'a, DB>>(
        &mut self,
        sql: Query<'a, DB, A>,
    ) ->  impl Future<Output = Result<Record<DATA>, C3p0Error>>;

    fn count_all<DATA: Data>(&mut self) -> impl Future<Output = Result<u64, C3p0Error>>;

    fn exists_by_id<DATA: Data>(&mut self, id: u64) -> impl Future<Output = Result<bool, C3p0Error>>;

    fn fetch_all<DATA: Data>(&mut self) -> impl Future<Output = Result<Vec<Record<DATA>>, C3p0Error>>;

    fn fetch_one_optional_by_id<DATA: Data>(
        &mut self,
        id: u64,
    ) -> impl Future<Output = Result<Option<Record<DATA>>, C3p0Error>>;

    fn fetch_one_by_id<DATA: Data>(&mut self, id: u64) -> impl Future<Output = Result<Record<DATA>, C3p0Error>> + Send;

    fn delete<DATA: Data>(
        &mut self,
        record: Record<DATA>,
    ) -> impl Future<Output = Result<Record<DATA>, C3p0Error>>;

    fn delete_all<DATA: Data>(&mut self) -> impl Future<Output = Result<u64, C3p0Error>>;

    fn delete_by_id<DATA: Data>(&mut self, id: u64) -> impl Future<Output = Result<u64, C3p0Error>>;

    fn update<DATA: Data>(
        &mut self,
        record: Record<DATA>,
    ) -> impl Future<Output = Result<Record<DATA>, C3p0Error>>;

    fn save<DATA: Data>(&mut self, record: NewRecord<DATA>) -> impl Future<Output = Result<Record<DATA>, C3p0Error>> + Send;

}

