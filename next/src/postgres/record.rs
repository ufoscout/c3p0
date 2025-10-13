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
        static QUERY: OnceLock::<String> = OnceLock::new();
        let query = QUERY.get_or_init(|| format!(
            "SELECT COUNT(*) FROM {}",
            DATA::TABLE_NAME,
        ));

        sqlx::query(&query)
            .fetch_one(tx)
            .await
            .and_then(|row| row.try_get(0))
            .map_err(into_c3p0_error)
            .map(|val: i64| val as u64)
    }

    async fn exists_by_id(tx: &mut PgConnection, id: u64) -> Result<bool, C3p0Error> {
                static QUERY: OnceLock::<String> = OnceLock::new();
        let query = QUERY.get_or_init(|| format!(
            "SELECT EXISTS (SELECT 1 FROM {} WHERE id = $1)",
            DATA::TABLE_NAME,
        ));

        sqlx::query(query)
            .bind(id as i64)
            .fetch_one(tx)
            .await
            .and_then(|row| row.try_get(0))
            .map_err(into_c3p0_error)
    }

    async fn fetch_all(tx: &mut PgConnection) -> Result<Vec<Record<DATA>>, C3p0Error> {
                        static QUERY: OnceLock::<String> = OnceLock::new();
        let query = QUERY.get_or_init(|| format!(
            "{} ORDER BY id ASC",
            Record::<DATA>::select_query_base(),
        ));

        Self::fetch_all_with_sql(tx, sqlx::query(query))
            .await
    }

    async fn fetch_one_optional_by_id(
        tx: &mut PgConnection,
        id: u64,
    ) -> Result<Option<Record<DATA>>, C3p0Error> {
                                static QUERY: OnceLock::<String> = OnceLock::new();
        let query = QUERY.get_or_init(|| format!(
            "{} WHERE id = $1 LIMIT 1",
            Record::<DATA>::select_query_base(),
        ));

        let query =         sqlx::query(query)
            .bind(id as i64);
        Self::fetch_one_optional_with_sql(tx, query).await
    }

    async fn fetch_one_by_id(
        tx: &mut PgConnection,
        id: u64,
    ) -> Result<Record<DATA>, C3p0Error> {
                                static QUERY: OnceLock::<String> = OnceLock::new();
        let query = QUERY.get_or_init(|| format!(
            "{} WHERE id = $1 LIMIT 1",
            Record::<DATA>::select_query_base(),
        ));

        let query =         sqlx::query(query)
            .bind(id as i64);
        Self::fetch_one_with_sql(tx, query).await
    }

    async fn delete(
        self,
        tx: &mut PgConnection,
    ) -> Result<Record<DATA>, C3p0Error> {
        static QUERY: OnceLock::<String> = OnceLock::new();
        let query = QUERY.get_or_init(|| format!(
            "DELETE FROM {} WHERE id = $1 AND version = $2",
            DATA::TABLE_NAME,
        ));

        let result = sqlx::query(query)
            .bind(self.id as i64)
            .bind(self.version)
            .execute(tx)
            .await
            .map_err(into_c3p0_error)?
            .rows_affected();

        if result == 0 {
            return Err(C3p0Error::OptimisticLockError {
                cause: format!(
                    "Cannot delete data in table [{}] with id [{:?}], version [{}]: data was changed!",
                    DATA::TABLE_NAME, self.id, self.version
                ),
            });
        }

        Ok(self)
    }

    async fn delete_all(tx: &mut PgConnection) -> Result<u64, C3p0Error> {
        static QUERY: OnceLock::<String> = OnceLock::new();
        let query = QUERY.get_or_init(|| format!("DELETE FROM {}",DATA::TABLE_NAME));

        sqlx::query(query)
            .execute(tx)
            .await
            .map_err(into_c3p0_error)
            .map(|done| done.rows_affected())
    }

    async fn delete_by_id(tx: &mut PgConnection, id: u64) -> Result<u64, C3p0Error> {
        static QUERY: OnceLock::<String> = OnceLock::new();
        let query = QUERY.get_or_init(|| format!("DELETE FROM {} WHERE id = $1",DATA::TABLE_NAME));

        sqlx::query(query)
            .bind(id as i64)
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