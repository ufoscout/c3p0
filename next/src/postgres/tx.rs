use sqlx::{PgConnection, Postgres};

use crate::{C3p0Error, Data, DbRead, DbWrite, NewRecord, Record, Tx};


impl Tx<Postgres> for PgConnection {
    async fn fetch_all_with_sql<'a, DATA: Data, A: 'a + Send + sqlx::IntoArguments<'a, Postgres>>(
        &mut self,
        sql: sqlx::query::Query<'a, Postgres, A>,
    ) -> Result<Vec<Record<DATA>>, C3p0Error> {
        Record::<DATA>::fetch_all_with_sql(self, sql).await
    }

    async fn fetch_one_optional_with_sql<'a, DATA: Data, A: 'a + Send + sqlx::IntoArguments<'a, Postgres>>(
        &mut self,
        sql: sqlx::query::Query<'a, Postgres, A>,
    ) -> Result<Option<Record<DATA>>, C3p0Error> {
         Record::<DATA>::fetch_one_optional_with_sql(self, sql).await
    }

    async fn fetch_one_with_sql<'a, DATA: Data, A: 'a + Send + sqlx::IntoArguments<'a, Postgres>>(
        &mut self,
        sql: sqlx::query::Query<'a, Postgres, A>,
    ) ->  Result<Record<DATA>, C3p0Error> {
        Record::<DATA>::fetch_one_with_sql(self, sql).await
    }

    async fn count_all<DATA: Data>(&mut self) -> Result<u64, C3p0Error> {
        Record::<DATA>::count_all(self).await
    }

    async fn exists_by_id<DATA: Data>(&mut self, id: u64) -> Result<bool, C3p0Error> {
        Record::<DATA>::exists_by_id(self, id).await
    }

    async fn fetch_all<DATA: Data>(&mut self) -> Result<Vec<Record<DATA>>, C3p0Error> {
        Record::<DATA>::fetch_all(self).await
    }

    async fn fetch_one_optional_by_id<DATA: Data>(
        &mut self,
        id: u64,
    ) -> Result<Option<Record<DATA>>, C3p0Error> {
        Record::<DATA>::fetch_one_optional_by_id(self, id).await
    }

    async fn fetch_one_by_id<DATA: Data>(&mut self, id: u64) -> Result<Record<DATA>, C3p0Error> {
        Record::<DATA>::fetch_one_by_id(self, id).await
    }

    async fn delete<DATA: Data>(
        &mut self,
        record: Record<DATA>,
    ) -> Result<Record<DATA>, C3p0Error> {
        record.delete(self).await
    }

    async fn delete_all<DATA: Data>(&mut self) -> Result<u64, C3p0Error> {
        Record::<DATA>::delete_all(self).await
    }

    async fn delete_by_id<DATA: Data>(&mut self, id: u64) -> Result<u64, C3p0Error> {
        Record::<DATA>::delete_by_id(self, id).await
    }

    async fn update<DATA: Data>(
        &mut self,
        record: Record<DATA>,
    ) -> Result<Record<DATA>, C3p0Error> {
        record.update(self).await
    }

    async fn save<DATA: Data>(&mut self, record: NewRecord<DATA>) -> Result<Record<DATA>, C3p0Error> {
        record.save(self).await
    }
}