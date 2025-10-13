use std::sync::OnceLock;

use sqlx::query::Query;
use sqlx::IntoArguments;
use sqlx::PgConnection;
use sqlx::Postgres;
use sqlx::Row;
use crate::codec::Codec;
use crate::error::into_c3p0_error;
use crate::record::row_to_record_with_index;
use crate::time::get_current_epoch_millis;
use crate::{error::C3p0Error, record::{Data, NewRecord, Record, DbRead, DbWrite}};


impl <DATA: Data> DbRead<Postgres, DATA> for Record<DATA> {

    async fn fetch_all_with_sql<'a, A: 'a + Send + IntoArguments<'a, Postgres>>(
        tx: &mut PgConnection,
        sql: Query<'a, Postgres, A>,
    ) -> Result<Vec<Record<DATA>>, C3p0Error> {
        sql.fetch_all(tx)
            .await
            .map_err(into_c3p0_error)?
            .iter()
            .map(|row| row_to_record_with_index(row, 0,1,2,3,4))
            .collect::<Result<Vec<_>, C3p0Error>>()
    }

    async fn fetch_one_optional_with_sql<'a, A: 'a + Send + IntoArguments<'a, Postgres>>(
        tx: &mut PgConnection,
        sql: Query<'a, Postgres, A>,
    ) -> Result<Option<Record<DATA>>, C3p0Error> {
        sql.fetch_optional(tx)
            .await
            .map_err(into_c3p0_error)?
            .map(|row| row_to_record_with_index(&row, 0,1,2,3,4))
            .transpose()
    }

    async fn fetch_one_with_sql<'a, A: 'a + Send + IntoArguments<'a, Postgres>>(
        tx: &mut PgConnection,
        sql: Query<'a, Postgres, A>,
    ) ->  Result<Record<DATA>, C3p0Error> {
            sql.fetch_one(tx)
                .await
                .map_err(into_c3p0_error)
                .and_then(|row| row_to_record_with_index(&row, 0,1,2,3,4))

    }

    async fn count_all(tx: &mut PgConnection) -> Result<u64, C3p0Error> {
        sqlx::query(&self.queries.count_all_sql_query)
            .fetch_one(tx)
            .await
            .and_then(|row| row.try_get(0))
            .map_err(into_c3p0_error)
            .map(|val: i64| val as u64)
    }

    async fn exists_by_id(tx: &mut PgConnection, id: u64) -> Result<bool, C3p0Error> {
        Self::query_with_id(&self.queries.exists_by_id_sql_query, id)
            .fetch_one(tx)
            .await
            .and_then(|row| row.try_get(0))
            .map_err(into_c3p0_error)
    }

    async fn fetch_all(tx: &mut PgConnection) -> Result<Vec<Record<DATA>>, C3p0Error> {
        Self::fetch_all_with_sql(tx, sqlx::query(&self.queries.find_all_sql_query))
            .await
    }

    async fn fetch_one_optional_by_id(
        tx: &mut PgConnection,
        id: u64,
    ) -> Result<Option<Record<DATA>>, C3p0Error> {
        let query = self.query_with_id(&self.queries.find_by_id_sql_query, id);
        Self::fetch_one_optional_with_sql(tx, query).await
    }

    async fn fetch_one_by_id(
        tx: &mut PgConnection,
        id: u64,
    ) -> Result<Record<DATA>, C3p0Error> {
        let query = self.query_with_id(&self.queries.find_by_id_sql_query, id);
        Self::fetch_one_with_sql(tx, query).await
    }

    async fn delete(
        self,
        tx: &mut PgConnection,
    ) -> Result<Record<DATA>, C3p0Error> {
        let result = self
            .query_with_id(&self.queries.delete_sql_query, &obj.id)
            .bind(obj.version as SqlxVersionType)
            .execute(tx)
            .await
            .map_err(into_c3p0_error)?
            .rows_affected();

        if result == 0 {
            return Err(C3p0Error::OptimisticLockError {
                cause: format!(
                    "Cannot delete data in table [{}] with id [{:?}], version [{}]: data was changed!",
                    &self.queries.qualified_table_name, &obj.id, &obj.version
                ),
            });
        }

        Ok(obj)
    }

    async fn delete_all(tx: &mut PgConnection) -> Result<u64, C3p0Error> {
        sqlx::query(&self.queries.delete_all_sql_query)
            .execute(tx)
            .await
            .map_err(into_c3p0_error)
            .map(|done| done.rows_affected())
    }

    async fn delete_by_id(tx: &mut PgConnection, id: u64) -> Result<u64, C3p0Error> {
        self.query_with_id(&self.queries.delete_by_id_sql_query, id)
            .execute(tx)
            .await
            .map_err(into_c3p0_error)
            .map(|done| done.rows_affected())
    }

}

impl <DATA: Data> DbWrite<Postgres, DATA> for NewRecord<DATA> {

    async fn save(self, tx: &mut PgConnection) -> Result<Record<DATA>, C3p0Error> {
        static QUERY: OnceLock::<String> = OnceLock::new();
        let query = QUERY.get_or_init(|| format!(
            "INSERT INTO {} (version, create_epoch_millis, update_epoch_millis, data) VALUES ($1, $2, $2, $3) RETURNING id",
            DATA::TABLE_NAME,
        ));

        let data_encoded = DATA::CODEC::encode(self.data);
        let json_data = serde_json::to_value(&data_encoded)?;
        let data = DATA::CODEC::decode(data_encoded);

        let create_epoch_millis = get_current_epoch_millis();

        let id = sqlx::query(&query)
                .bind(0)
                .bind(create_epoch_millis)
                .bind(json_data)
                .fetch_one(tx)
                .await
                .map_err(into_c3p0_error)
                .and_then(|row| {
                    row.try_get(&0)
                    .map_err(|err| C3p0Error::RowMapperError {
                        cause: format!("Row contains no values for id index. Err: {err:?}"),
                    })
                    .map(|id: i64| id as u64)
                })?;

        Ok(Record {
            id,
            version: 0,
            data,
            create_epoch_millis,
            update_epoch_millis: create_epoch_millis,
        })
    }

}