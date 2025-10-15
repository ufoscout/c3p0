
use crate::codec::Codec;
use crate::error::into_c3p0_error;
use crate::record::row_to_record_with_index;
use crate::time::get_current_epoch_millis;
use crate::{
    error::C3p0Error,
    record::{DataType, DbOps, DbSave, NewRecord, Record},
};

use sqlx::{Database, IntoArguments};
use sqlx::PgConnection;
use sqlx::Postgres;
use sqlx::Row;
use sqlx::query::{Query, QueryAs};

impl<DATA: DataType> DbOps<Postgres, DATA> for Record<DATA> {

    fn query_with(
        sql: &str,
    ) -> QueryAs<'_, Postgres, Record<DATA>, <Postgres as Database>::Arguments> {
        let query = format!("{} {}", select_query_base(DATA::TABLE_NAME), sql);
        sqlx::query_as(sqlx::AssertSqlSafe(query))
    }

    async fn fetch_all_with_sql<A: IntoArguments<Postgres>>(
        tx: &mut PgConnection,
        sql: Query<'_, Postgres, A>,
    ) -> Result<Vec<Record<DATA>>, C3p0Error> {
        sql.fetch_all(tx)
            .await
            .map_err(into_c3p0_error)?
            .iter()
            .map(|row| row_to_record_with_index(row, 0, 1, 2, 3, 4))
            .collect::<Result<Vec<_>, C3p0Error>>()
    }

    async fn fetch_one_optional_with_sql<A: IntoArguments<Postgres>>(
        tx: &mut PgConnection,
        sql: Query<'_, Postgres, A>,
    ) -> Result<Option<Record<DATA>>, C3p0Error> {
        sql.fetch_optional(tx)
            .await
            .map_err(into_c3p0_error)?
            .map(|row| row_to_record_with_index(&row, 0, 1, 2, 3, 4))
            .transpose()
    }

    async fn fetch_one_with_sql<A: IntoArguments<Postgres>>(
        tx: &mut PgConnection,
        sql: Query<'_, Postgres, A>,
    ) -> Result<Record<DATA>, C3p0Error> {
        sql.fetch_one(tx)
            .await
            .map_err(into_c3p0_error)
            .and_then(|row| row_to_record_with_index(&row, 0, 1, 2, 3, 4))
    }

    async fn count_all(tx: &mut PgConnection) -> Result<u64, C3p0Error> {
        let query = format!("SELECT COUNT(*) FROM {}", DATA::TABLE_NAME,);

        sqlx::query(sqlx::AssertSqlSafe(query))
            .fetch_one(tx)
            .await
            .and_then(|row| row.try_get(0))
            .map_err(into_c3p0_error)
            .map(|val: i64| val as u64)
    }

    async fn exists_by_id(tx: &mut PgConnection, id: u64) -> Result<bool, C3p0Error> {
        let query = format!(
            "SELECT EXISTS (SELECT 1 FROM {} WHERE id = $1)",
            DATA::TABLE_NAME,
        );

        sqlx::query(sqlx::AssertSqlSafe(query))
            .bind(id as i64)
            .fetch_one(tx)
            .await
            .and_then(|row| row.try_get(0))
            .map_err(into_c3p0_error)
    }

    async fn fetch_all(tx: &mut PgConnection) -> Result<Vec<Record<DATA>>, C3p0Error> {
        Self::query_with(" ORDER BY id ASC")
            .fetch_all(tx)
            .await
            .map_err(into_c3p0_error)
    }

    async fn fetch_one_optional_by_id(
        tx: &mut PgConnection,
        id: u64,
    ) -> Result<Option<Record<DATA>>, C3p0Error> {
        Self::query_with(" WHERE id = $1 LIMIT 1")
            .bind(id as i64)
            .fetch_optional(tx)
            .await
            .map_err(into_c3p0_error)
    }

    async fn fetch_one_by_id(tx: &mut PgConnection, id: u64) -> Result<Record<DATA>, C3p0Error> {
        Self::query_with(" WHERE id = $1 LIMIT 1")
            .bind(id as i64)
            .fetch_one(tx)
            .await
            .map_err(into_c3p0_error)
    }

    async fn delete(self, tx: &mut PgConnection) -> Result<Record<DATA>, C3p0Error> {
        let query = format!(
            "DELETE FROM {} WHERE id = $1 AND version = $2",
            DATA::TABLE_NAME,
        );

        let result = sqlx::query(sqlx::AssertSqlSafe(query))
            .bind(self.id as i64)
            .bind(self.version as i32)
            .execute(tx)
            .await
            .map_err(into_c3p0_error)?
            .rows_affected();

        if result == 0 {
            return Err(C3p0Error::OptimisticLockError {
                cause: format!(
                    "Cannot delete data in table [{}] with id [{:?}], version [{}]: data was changed!",
                    DATA::TABLE_NAME,
                    self.id,
                    self.version
                ),
            });
        }

        Ok(self)
    }

    async fn delete_all(tx: &mut PgConnection) -> Result<u64, C3p0Error> {
        let query = format!("DELETE FROM {}", DATA::TABLE_NAME);

        sqlx::query(sqlx::AssertSqlSafe(query))
            .execute(tx)
            .await
            .map_err(into_c3p0_error)
            .map(|done| done.rows_affected())
    }

    async fn delete_by_id(tx: &mut PgConnection, id: u64) -> Result<u64, C3p0Error> {
        let query = format!("DELETE FROM {} WHERE id = $1", DATA::TABLE_NAME);

        sqlx::query(sqlx::AssertSqlSafe(query))
            .bind(id as i64)
            .execute(tx)
            .await
            .map_err(into_c3p0_error)
            .map(|done| done.rows_affected())
    }

    async fn update(mut self, tx: &mut PgConnection) -> Result<Record<DATA>, C3p0Error> {
        let query = format!(
            "UPDATE {} SET version = $1, update_epoch_millis = $2, data = $3 WHERE id = $4 AND version = $5",
            DATA::TABLE_NAME
        );

        let data_encoded = DATA::CODEC::encode(self.data);
        let json_data = serde_json::to_value(&data_encoded)?;
        let previous_version = self.version;

        self.data = DATA::CODEC::decode(data_encoded);
        self.version += 1;
        self.update_epoch_millis = get_current_epoch_millis();

        let result = {
            sqlx::query(sqlx::AssertSqlSafe(query))
                .bind(self.version as i32)
                .bind(self.update_epoch_millis)
                .bind(json_data)
                .bind(self.id as i64)
                .bind(previous_version as i32)
                .execute(tx)
                .await
                .map_err(into_c3p0_error)
                .map(|done| done.rows_affected())?
        };

        if result == 0 {
            return Err(C3p0Error::OptimisticLockError {
                cause: format!(
                    "Cannot update data in table [{}] with id [{:?}], version [{}]: data was changed!",
                    DATA::TABLE_NAME,
                    self.id,
                    &previous_version
                ),
            });
        }

        Ok(self)
    }
}

impl<DATA: DataType> DbSave<Postgres, DATA> for NewRecord<DATA> {
    async fn save(self, tx: &mut PgConnection) -> Result<Record<DATA>, C3p0Error> {
        let query = format!(
            "INSERT INTO {} (version, create_epoch_millis, update_epoch_millis, data) VALUES ($1, $2, $2, $3) RETURNING id",
            DATA::TABLE_NAME,
        );

        let data_encoded = DATA::CODEC::encode(self.data);
        let json_data = serde_json::to_value(&data_encoded)?;
        let data = DATA::CODEC::decode(data_encoded);

        let create_epoch_millis = get_current_epoch_millis();

        let id = sqlx::query(sqlx::AssertSqlSafe(query))
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

/// Returns a SQL query string to select all columns from the database table.
fn select_query_base(table_name: &str) -> String {
    format!(
        "SELECT id, version, create_epoch_millis, update_epoch_millis, data FROM {}",
        table_name
    )
}
