use sqlx::{Sqlite, SqliteConnection};

use crate::{C3p0Error, DataType, DbOps, DbSave, NewRecord, Record, Tx, WithData};

impl Tx for SqliteConnection {
    type DB = Sqlite;

    async fn create_table_if_not_exists<DATA: WithData>(&mut self) -> Result<(), C3p0Error> {
        let query = format!(
            r#"
                CREATE TABLE IF NOT EXISTS {} (
                    id integer primary key autoincrement,
                    version integer not null,
                    create_epoch_millis integer not null,
                    update_epoch_millis integer not null,
                    data JSON
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
        _cascade: bool,
    ) -> Result<(), C3p0Error> {
        let query = format!(
            "DROP TABLE IF EXISTS {}",
            <DATA::DATA as DataType>::TABLE_NAME
        );
        Ok(sqlx::query(sqlx::AssertSqlSafe(query))
            .execute(self)
            .await
            .map(|_| ())?)
    }

    async fn count_all<DATA: WithData>(&mut self) -> Result<u64, C3p0Error> {
        <Record<DATA::DATA> as DbOps<Sqlite, DATA::DATA>>::count_all(self).await
    }

    async fn exists_by_id<DATA: WithData>(&mut self, id: u64) -> Result<bool, C3p0Error> {
        <Record<DATA::DATA> as DbOps<Sqlite, DATA::DATA>>::exists_by_id(self, id).await
    }

    async fn fetch_all<DATA: DataType>(&mut self) -> Result<Vec<Record<DATA>>, C3p0Error> {
        <Record<DATA> as DbOps<Sqlite, DATA>>::fetch_all(self).await
    }

    async fn fetch_one_optional_by_id<DATA: DataType>(
        &mut self,
        id: u64,
    ) -> Result<Option<Record<DATA>>, C3p0Error> {
        <Record<DATA> as DbOps<Sqlite, DATA>>::fetch_one_optional_by_id(self, id).await
    }

    async fn fetch_one_by_id<DATA: DataType>(
        &mut self,
        id: u64,
    ) -> Result<Record<DATA>, C3p0Error> {
        <Record<DATA> as DbOps<Sqlite, DATA>>::fetch_one_by_id(self, id).await
    }

    async fn delete<DATA: DataType>(
        &mut self,
        record: Record<DATA>,
    ) -> Result<Record<DATA>, C3p0Error> {
        <Record<DATA> as DbOps<Sqlite, DATA>>::delete(record, self).await
    }

    async fn delete_all<DATA: WithData>(&mut self) -> Result<u64, C3p0Error> {
        <Record<DATA::DATA> as DbOps<Sqlite, DATA::DATA>>::delete_all(self).await
    }

    async fn delete_by_id<DATA: WithData>(&mut self, id: u64) -> Result<u64, C3p0Error> {
        <Record<DATA::DATA> as DbOps<Sqlite, DATA::DATA>>::delete_by_id(self, id).await
    }

    async fn update<DATA: DataType>(
        &mut self,
        record: Record<DATA>,
    ) -> Result<Record<DATA>, C3p0Error> {
        <Record<DATA> as DbOps<Sqlite, DATA>>::update(record, self).await
    }

    async fn save<DATA: DataType>(
        &mut self,
        record: NewRecord<DATA>,
    ) -> Result<Record<DATA>, C3p0Error> {
        <NewRecord<DATA> as DbSave<Sqlite, DATA>>::save(record, self).await
    }
}
