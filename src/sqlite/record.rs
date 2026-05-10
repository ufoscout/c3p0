use chrono::{DateTime, Utc};

use crate::codec::Codec;
use crate::{
    error::C3p0Error,
    record::{DataType, DbOps, DbSave, NewRecord, Record},
};
use sqlx::Database;
use sqlx::FromRow;
use sqlx::Row;
use sqlx::Sqlite;
use sqlx::SqliteConnection;
use sqlx::query::QueryAs;
use sqlx::sqlite::SqliteRow;

/// SQL expression returning the current timestamp as ISO-8601 UTC text, computed by the DB
/// server. Used to populate `create_time` / `update_time` from the DB clock instead of the
/// writer's local clock (avoids cross-machine skew). SQLite's `'now'` is evaluated once per
/// `sqlite3_step()` call (i.e. **per statement**, not per transaction as Postgres'
/// `CURRENT_TIMESTAMP` is), so successive writes inside one `pool.transaction(...)` block
/// receive successive values. The format `YYYY-MM-DDTHH:MM:SS.sssZ` matches sqlx-sqlite's
/// `%FT%T%.fZ` decode pattern for `DateTime<Utc>`. Resolution: 1 ms (SQLite's `%f` modifier
/// yields three fractional digits).
const NOW_EXPR: &str = "strftime('%Y-%m-%dT%H:%M:%fZ', 'now')";

impl<DATA: DataType> FromRow<'_, SqliteRow> for Record<DATA> {
    fn from_row(row: &SqliteRow) -> Result<Self, sqlx::Error> {
        let id: i64 = row.try_get(0)?;
        let version: i64 = row.try_get(1)?;
        let create_time: DateTime<Utc> = row.try_get(2)?;
        let update_time: DateTime<Utc> = row.try_get(3)?;
        let sqlx::types::Json(data): sqlx::types::Json<DATA::CODEC> = row.try_get(4)?;

        Ok(Record {
            id: id as u64,
            version: version as u64,
            data: DATA::CODEC::decode(data),
            create_time,
            update_time,
        })
    }
}

impl<DATA: DataType> DbOps<Sqlite, DATA> for Record<DATA> {
    fn query_with_tail(
        tail: &str,
    ) -> QueryAs<'_, Sqlite, Record<DATA>, <Sqlite as Database>::Arguments> {
        let query = format!(
            "{} {}",
            <Self as DbOps<Sqlite, DATA>>::select_query_base(),
            tail
        );
        sqlx::query_as(sqlx::AssertSqlSafe(query))
    }

    async fn count_all(tx: &mut SqliteConnection) -> Result<u64, C3p0Error> {
        let query = format!("SELECT COUNT(*) FROM {}", DATA::TABLE_NAME,);

        Ok(sqlx::query(sqlx::AssertSqlSafe(query))
            .fetch_one(tx)
            .await
            .and_then(|row| row.try_get(0))
            .map(|val: i64| val as u64)?)
    }

    async fn exists_by_id(tx: &mut SqliteConnection, id: u64) -> Result<bool, C3p0Error> {
        let query = format!(
            "SELECT EXISTS (SELECT 1 FROM {} WHERE id = ?)",
            DATA::TABLE_NAME,
        );

        Ok(sqlx::query(sqlx::AssertSqlSafe(query))
            .bind(id as i64)
            .fetch_one(tx)
            .await
            .and_then(|row| row.try_get(0))?)
    }

    async fn fetch_all(
        tx: &mut SqliteConnection,
        offset: u64,
        limit: Option<u64>,
    ) -> Result<Vec<Record<DATA>>, C3p0Error> {
        let query = match limit {
            Some(limit) => Self::query_with_tail("ORDER BY id ASC LIMIT ? OFFSET ?")
                .bind(limit as i64)
                .bind(offset as i64),
            // SQLite treats a negative LIMIT as "no upper bound" (per its docs).
            None => Self::query_with_tail("ORDER BY id ASC LIMIT -1 OFFSET ?").bind(offset as i64),
        };
        Ok(query.fetch_all(tx).await?)
    }

    async fn fetch_one_optional_by_id(
        tx: &mut SqliteConnection,
        id: u64,
    ) -> Result<Option<Record<DATA>>, C3p0Error> {
        Ok(Self::query_with_tail("WHERE id = ? LIMIT 1")
            .bind(id as i64)
            .fetch_optional(tx)
            .await?)
    }

    async fn fetch_one_by_id(
        tx: &mut SqliteConnection,
        id: u64,
    ) -> Result<Record<DATA>, C3p0Error> {
        Ok(Self::query_with_tail("WHERE id = ? LIMIT 1")
            .bind(id as i64)
            .fetch_one(tx)
            .await?)
    }

    async fn delete(self, tx: &mut SqliteConnection) -> Result<Record<DATA>, C3p0Error> {
        let query = format!(
            "DELETE FROM {} WHERE id = ? AND version = ?",
            DATA::TABLE_NAME,
        );

        let result = sqlx::query(sqlx::AssertSqlSafe(query))
            .bind(self.id as i64)
            .bind(self.version as i64)
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

    async fn delete_all(tx: &mut SqliteConnection) -> Result<u64, C3p0Error> {
        let query = format!("DELETE FROM {}", DATA::TABLE_NAME);

        Ok(sqlx::query(sqlx::AssertSqlSafe(query))
            .execute(tx)
            .await
            .map(|done| done.rows_affected())?)
    }

    async fn delete_by_id(tx: &mut SqliteConnection, id: u64) -> Result<u64, C3p0Error> {
        let query = format!("DELETE FROM {} WHERE id = ?", DATA::TABLE_NAME);

        Ok(sqlx::query(sqlx::AssertSqlSafe(query))
            .bind(id as i64)
            .execute(tx)
            .await
            .map(|done| done.rows_affected())?)
    }

    async fn update(mut self, tx: &mut SqliteConnection) -> Result<Record<DATA>, C3p0Error> {
        let query = format!(
            "UPDATE {} SET version = ?, update_time = {NOW_EXPR}, data = ? \
             WHERE id = ? AND version = ? RETURNING update_time",
            DATA::TABLE_NAME
        );

        let data_encoded = DATA::CODEC::encode(self.data);
        let previous_version = self.version;
        let new_version = previous_version + 1;

        let row = sqlx::query(sqlx::AssertSqlSafe(query))
            .bind(new_version as i64)
            .bind(sqlx::types::Json(&data_encoded))
            .bind(self.id as i64)
            .bind(previous_version as i64)
            .fetch_optional(tx)
            .await?;

        let Some(row) = row else {
            return Err(C3p0Error::OptimisticLockError {
                cause: format!(
                    "Cannot update data in table [{}] with id [{:?}], version [{}]: data was changed!",
                    DATA::TABLE_NAME,
                    self.id,
                    &previous_version
                ),
            });
        };

        self.data = DATA::CODEC::decode(data_encoded);
        self.version = new_version;
        self.update_time = row.try_get(0)?;
        Ok(self)
    }
}

impl<DATA: DataType> DbSave<Sqlite, DATA> for NewRecord<DATA> {
    async fn save(self, tx: &mut SqliteConnection) -> Result<Record<DATA>, C3p0Error> {
        let query = format!(
            "WITH ts AS (SELECT {NOW_EXPR} AS v) \
             INSERT INTO {} (version, create_time, update_time, data) \
             SELECT ?, ts.v, ts.v, ? FROM ts \
             RETURNING id, create_time",
            DATA::TABLE_NAME,
        );

        let data_encoded = DATA::CODEC::encode(self.data);

        let row = sqlx::query(sqlx::AssertSqlSafe(query))
            .bind(0_i64)
            .bind(sqlx::types::Json(&data_encoded))
            .fetch_one(tx)
            .await?;
        let id: i64 = row.try_get(0)?;
        let create_time: DateTime<Utc> = row.try_get(1)?;
        let data = DATA::CODEC::decode(data_encoded);

        Ok(Record {
            id: id as u64,
            version: 0,
            data,
            create_time,
            update_time: create_time,
        })
    }
}
