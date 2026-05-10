use chrono::{DateTime, Utc};

use crate::codec::Codec;
use crate::{
    error::C3p0Error,
    record::{DataType, DbOps, DbSave, NewRecord, Record},
};
use sqlx::Database;
use sqlx::FromRow;
use sqlx::MySql;
use sqlx::MySqlConnection;
use sqlx::Row;
use sqlx::mysql::MySqlRow;
use sqlx::query::QueryAs;

/// SQL expression returning the current `TIMESTAMP(3)` (millisecond precision) computed by
/// the DB server. Used to populate `create_time` / `update_time` from the DB clock instead
/// of the writer's local clock (avoids cross-machine skew). MySQL/MariaDB evaluate
/// `CURRENT_TIMESTAMP(3)` once **per statement** — not once per transaction as Postgres'
/// `CURRENT_TIMESTAMP` does — so successive writes inside one `pool.transaction(...)` block
/// receive successive values, even though both columns within a single INSERT share one
/// value. Resolution: 1 ms.
const NOW_EXPR: &str = "CURRENT_TIMESTAMP(3)";

impl<DATA: DataType> FromRow<'_, MySqlRow> for Record<DATA> {
    fn from_row(row: &MySqlRow) -> Result<Self, sqlx::Error> {
        let id: u64 = row.try_get(0)?;
        let version: u64 = row.try_get(1)?;
        let create_time: DateTime<Utc> = row.try_get(2)?;
        let update_time: DateTime<Utc> = row.try_get(3)?;
        let sqlx::types::Json(data): sqlx::types::Json<DATA::CODEC> = row.try_get(4)?;

        Ok(Record {
            id,
            version,
            data: DATA::CODEC::decode(data),
            create_time,
            update_time,
        })
    }
}

impl<DATA: DataType> DbOps<MySql, DATA> for Record<DATA> {
    fn query_with_tail(
        tail: &str,
    ) -> QueryAs<'_, MySql, Record<DATA>, <MySql as Database>::Arguments> {
        let query = format!(
            "{} {}",
            <Self as DbOps<MySql, DATA>>::select_query_base(),
            tail
        );
        sqlx::query_as(sqlx::AssertSqlSafe(query))
    }

    async fn count_all(tx: &mut MySqlConnection) -> Result<u64, C3p0Error> {
        let query = format!("SELECT COUNT(*) FROM {}", DATA::TABLE_NAME,);

        Ok(sqlx::query(sqlx::AssertSqlSafe(query))
            .fetch_one(tx)
            .await
            .and_then(|row| row.try_get(0))
            .map(|val: i64| val as u64)?)
    }

    async fn exists_by_id(tx: &mut MySqlConnection, id: u64) -> Result<bool, C3p0Error> {
        let query = format!(
            "SELECT EXISTS (SELECT 1 FROM {} WHERE id = ?)",
            DATA::TABLE_NAME,
        );

        Ok(sqlx::query(sqlx::AssertSqlSafe(query))
            .bind(id)
            .fetch_one(tx)
            .await
            .and_then(|row| row.try_get(0))?)
    }

    async fn fetch_all(
        tx: &mut MySqlConnection,
        offset: u64,
        limit: Option<u64>,
    ) -> Result<Vec<Record<DATA>>, C3p0Error> {
        let query = match limit {
            Some(limit) => Self::query_with_tail("ORDER BY id ASC LIMIT ? OFFSET ?")
                .bind(limit)
                .bind(offset),
            // MySQL requires LIMIT to use OFFSET; u64::MAX is the documented sentinel for "no limit".
            None => Self::query_with_tail("ORDER BY id ASC LIMIT 18446744073709551615 OFFSET ?")
                .bind(offset),
        };
        Ok(query.fetch_all(tx).await?)
    }

    async fn fetch_one_optional_by_id(
        tx: &mut MySqlConnection,
        id: u64,
    ) -> Result<Option<Record<DATA>>, C3p0Error> {
        Ok(Self::query_with_tail("WHERE id = ? LIMIT 1")
            .bind(id)
            .fetch_optional(tx)
            .await?)
    }

    async fn fetch_one_by_id(tx: &mut MySqlConnection, id: u64) -> Result<Record<DATA>, C3p0Error> {
        Ok(Self::query_with_tail("WHERE id = ? LIMIT 1")
            .bind(id)
            .fetch_one(tx)
            .await?)
    }

    async fn delete(self, tx: &mut MySqlConnection) -> Result<Record<DATA>, C3p0Error> {
        let query = format!(
            "DELETE FROM {} WHERE id = ? AND version = ?",
            DATA::TABLE_NAME,
        );

        let result = sqlx::query(sqlx::AssertSqlSafe(query))
            .bind(self.id)
            .bind(self.version)
            .execute(tx)
            .await?
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

    async fn delete_all(tx: &mut MySqlConnection) -> Result<u64, C3p0Error> {
        let query = format!("DELETE FROM {}", DATA::TABLE_NAME);

        Ok(sqlx::query(sqlx::AssertSqlSafe(query))
            .execute(tx)
            .await
            .map(|done| done.rows_affected())?)
    }

    async fn delete_by_id(tx: &mut MySqlConnection, id: u64) -> Result<u64, C3p0Error> {
        let query = format!("DELETE FROM {} WHERE id = ?", DATA::TABLE_NAME);

        Ok(sqlx::query(sqlx::AssertSqlSafe(query))
            .bind(id)
            .execute(tx)
            .await
            .map(|done| done.rows_affected())?)
    }

    async fn update(mut self, tx: &mut MySqlConnection) -> Result<Record<DATA>, C3p0Error> {
        let query = format!(
            "UPDATE {} SET version = ?, update_time = {NOW_EXPR}, data = ? \
             WHERE id = ? AND version = ?",
            DATA::TABLE_NAME
        );
        let select_ts = format!("SELECT update_time FROM {} WHERE id = ?", DATA::TABLE_NAME,);

        let data_encoded = DATA::CODEC::encode(self.data);
        let previous_version = self.version;
        let new_version = previous_version + 1;

        let result = sqlx::query(sqlx::AssertSqlSafe(query))
            .bind(new_version)
            .bind(sqlx::types::Json(&data_encoded))
            .bind(self.id)
            .bind(previous_version)
            .execute(&mut *tx)
            .await
            .map(|done| done.rows_affected())?;

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

        self.data = DATA::CODEC::decode(data_encoded);
        self.version = new_version;
        self.update_time = sqlx::query(sqlx::AssertSqlSafe(select_ts))
            .bind(self.id)
            .fetch_one(tx)
            .await
            .and_then(|row| row.try_get(0))?;

        Ok(self)
    }
}

impl<DATA: DataType> DbSave<MySql, DATA> for NewRecord<DATA> {
    async fn save(self, tx: &mut MySqlConnection) -> Result<Record<DATA>, C3p0Error> {
        let query = format!(
            "INSERT INTO {} (version, create_time, update_time, data) \
             VALUES (?, {NOW_EXPR}, {NOW_EXPR}, ?)",
            DATA::TABLE_NAME,
        );
        let select_ts = format!("SELECT create_time FROM {} WHERE id = ?", DATA::TABLE_NAME,);

        let data_encoded = DATA::CODEC::encode(self.data);

        let id = sqlx::query(sqlx::AssertSqlSafe(query))
            .bind(0_u64)
            .bind(sqlx::types::Json(&data_encoded))
            .execute(&mut *tx)
            .await
            .map(|done| done.last_insert_id())?;
        let data = DATA::CODEC::decode(data_encoded);

        let create_time: DateTime<Utc> = sqlx::query(sqlx::AssertSqlSafe(select_ts))
            .bind(id)
            .fetch_one(tx)
            .await
            .and_then(|row| row.try_get(0))?;

        Ok(Record {
            id,
            version: 0,
            data,
            create_time,
            update_time: create_time,
        })
    }
}
