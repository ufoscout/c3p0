use sqlx::{PgConnection, Postgres};

use crate::{C3p0Error, DataType, DbOps, DbSave, NewRecord, Record, Tx, WithData};

impl Tx for PgConnection {
    type DB = Postgres;

    async fn create_table_if_not_exists<DATA: WithData>(&mut self) -> Result<(), C3p0Error> {
        let query = format!(
            r#"
                CREATE TABLE IF NOT EXISTS {} (
                    id bigserial primary key,
                    version bigint not null,
                    create_time TIMESTAMPTZ NOT NULL,
                    update_time TIMESTAMPTZ NOT NULL,
                    data JSONB NOT NULL
                )
                "#,
            <DATA::DATA as DataType>::TABLE_NAME,
        );

        Ok(sqlx::query(sqlx::AssertSqlSafe(query))
            .execute(self)
            .await
            .map(|_| ())?)
    }

    async fn drop_table_if_exists<DATA: WithData>(
        &mut self,
        cascade: bool,
    ) -> Result<(), C3p0Error> {
        let query = if cascade {
            format!(
                "DROP TABLE IF EXISTS {} CASCADE",
                <DATA::DATA as DataType>::TABLE_NAME
            )
        } else {
            format!(
                "DROP TABLE IF EXISTS {}",
                <DATA::DATA as DataType>::TABLE_NAME
            )
        };
        Ok(sqlx::query(sqlx::AssertSqlSafe(query))
            .execute(self)
            .await
            .map(|_| ())?)
    }

    async fn count_all<DATA: WithData>(&mut self) -> Result<u64, C3p0Error> {
        <Record<DATA::DATA> as DbOps<Postgres, DATA::DATA>>::count_all(self).await
    }

    async fn exists_by_id<DATA: WithData>(&mut self, id: u64) -> Result<bool, C3p0Error> {
        <Record<DATA::DATA> as DbOps<Postgres, DATA::DATA>>::exists_by_id(self, id).await
    }

    async fn fetch_all<DATA: WithData>(
        &mut self,
        offset: u64,
        limit: Option<u64>,
    ) -> Result<Vec<Record<DATA::DATA>>, C3p0Error> {
        <Record<DATA::DATA> as DbOps<Postgres, DATA::DATA>>::fetch_all(self, offset, limit).await
    }

    async fn fetch_one_optional_by_id<DATA: WithData>(
        &mut self,
        id: u64,
    ) -> Result<Option<Record<DATA::DATA>>, C3p0Error> {
        <Record<DATA::DATA> as DbOps<Postgres, DATA::DATA>>::fetch_one_optional_by_id(self, id)
            .await
    }

    async fn fetch_one_by_id<DATA: WithData>(
        &mut self,
        id: u64,
    ) -> Result<Record<DATA::DATA>, C3p0Error> {
        <Record<DATA::DATA> as DbOps<Postgres, DATA::DATA>>::fetch_one_by_id(self, id).await
    }

    async fn delete<DATA: DataType>(
        &mut self,
        record: Record<DATA>,
    ) -> Result<Record<DATA>, C3p0Error> {
        <Record<DATA> as DbOps<Postgres, DATA>>::delete(record, self).await
    }

    async fn delete_all<DATA: WithData>(&mut self) -> Result<u64, C3p0Error> {
        <Record<DATA::DATA> as DbOps<Postgres, DATA::DATA>>::delete_all(self).await
    }

    async fn delete_by_id<DATA: WithData>(&mut self, id: u64) -> Result<u64, C3p0Error> {
        <Record<DATA::DATA> as DbOps<Postgres, DATA::DATA>>::delete_by_id(self, id).await
    }

    async fn update<DATA: DataType>(
        &mut self,
        record: Record<DATA>,
    ) -> Result<Record<DATA>, C3p0Error> {
        <Record<DATA> as DbOps<Postgres, DATA>>::update(record, self).await
    }

    async fn save<DATA: DataType>(
        &mut self,
        record: NewRecord<DATA>,
    ) -> Result<Record<DATA>, C3p0Error> {
        <NewRecord<DATA> as DbSave<Postgres, DATA>>::save(record, self).await
    }
}
