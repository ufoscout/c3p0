use sqlx::{PgConnection, Postgres};

use crate::{C3p0Error, Data, DbRead, DbWrite, NewRecord, Record, Tx, error::into_c3p0_error};

impl Tx<Postgres> for PgConnection {
    async fn create_table_if_not_exists<DATA: Data>(&mut self) -> Result<(), C3p0Error> {
        let query = format!(
            r#"
                CREATE TABLE IF NOT EXISTS {} (
                    id bigserial primary key,
                    version int not null,
                    create_epoch_millis bigint not null,
                    update_epoch_millis bigint not null,
                    data JSONB
                )
                "#,
            DATA::TABLE_NAME,
        );

        sqlx::query(&query)
            .execute(self)
            .await
            .map_err(into_c3p0_error)
            .map(|_| ())
    }

    async fn drop_table_if_exists<DATA: Data>(&mut self, cascade: bool) -> Result<(), C3p0Error> {
        let query = if cascade {
            format!("DROP TABLE IF EXISTS {} CASCADE", DATA::TABLE_NAME)
        } else {
            format!("DROP TABLE IF EXISTS {}", DATA::TABLE_NAME)
        };
        sqlx::query(&query)
            .execute(self)
            .await
            .map_err(into_c3p0_error)
            .map(|_| ())
    }

    async fn fetch_all_with_sql<
        'a,
        DATA: Data,
        A: 'a + Send + sqlx::IntoArguments<'a, Postgres>,
    >(
        &mut self,
        sql: sqlx::query::Query<'a, Postgres, A>,
    ) -> Result<Vec<Record<DATA>>, C3p0Error> {
        <Record<DATA> as DbRead<Postgres, DATA>>::fetch_all_with_sql(self, sql).await
    }

    async fn fetch_one_optional_with_sql<
        'a,
        DATA: Data,
        A: 'a + Send + sqlx::IntoArguments<'a, Postgres>,
    >(
        &mut self,
        sql: sqlx::query::Query<'a, Postgres, A>,
    ) -> Result<Option<Record<DATA>>, C3p0Error> {
        <Record<DATA> as DbRead<Postgres, DATA>>::fetch_one_optional_with_sql(self, sql).await
    }

    async fn fetch_one_with_sql<
        'a,
        DATA: Data,
        A: 'a + Send + sqlx::IntoArguments<'a, Postgres>,
    >(
        &mut self,
        sql: sqlx::query::Query<'a, Postgres, A>,
    ) -> Result<Record<DATA>, C3p0Error> {
        <Record<DATA> as DbRead<Postgres, DATA>>::fetch_one_with_sql(self, sql).await
    }

    async fn count_all<DATA: Data>(&mut self) -> Result<u64, C3p0Error> {
        <Record<DATA> as DbRead<Postgres, DATA>>::count_all(self).await
    }

    async fn exists_by_id<DATA: Data>(&mut self, id: u64) -> Result<bool, C3p0Error> {
        <Record<DATA> as DbRead<Postgres, DATA>>::exists_by_id(self, id).await
    }

    async fn fetch_all<DATA: Data>(&mut self) -> Result<Vec<Record<DATA>>, C3p0Error> {
        <Record<DATA> as DbRead<Postgres, DATA>>::fetch_all(self).await
    }

    async fn fetch_one_optional_by_id<DATA: Data>(
        &mut self,
        id: u64,
    ) -> Result<Option<Record<DATA>>, C3p0Error> {
        <Record<DATA> as DbRead<Postgres, DATA>>::fetch_one_optional_by_id(self, id).await
    }

    async fn fetch_one_by_id<DATA: Data>(&mut self, id: u64) -> Result<Record<DATA>, C3p0Error> {
        <Record<DATA> as DbRead<Postgres, DATA>>::fetch_one_by_id(self, id).await
    }

    async fn delete<DATA: Data>(
        &mut self,
        record: Record<DATA>,
    ) -> Result<Record<DATA>, C3p0Error> {
        <Record<DATA> as DbRead<Postgres, DATA>>::delete(record, self).await
    }

    async fn delete_all<DATA: Data>(&mut self) -> Result<u64, C3p0Error> {
        <Record<DATA> as DbRead<Postgres, DATA>>::delete_all(self).await
    }

    async fn delete_by_id<DATA: Data>(&mut self, id: u64) -> Result<u64, C3p0Error> {
        <Record<DATA> as DbRead<Postgres, DATA>>::delete_by_id(self, id).await
    }

    async fn update<DATA: Data>(
        &mut self,
        record: Record<DATA>,
    ) -> Result<Record<DATA>, C3p0Error> {
        <Record<DATA> as DbRead<Postgres, DATA>>::update(record, self).await
    }

    async fn save<DATA: Data>(
        &mut self,
        record: NewRecord<DATA>,
    ) -> Result<Record<DATA>, C3p0Error> {
        <NewRecord<DATA> as DbWrite<Postgres, DATA>>::save(record, self).await
    }
}
