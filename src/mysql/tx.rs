use sqlx::{MySql, MySqlConnection};

use crate::{C3p0Error, DataType, DbOps, DbSave, NewRecord, Record, Tx, WithData};

impl Tx for MySqlConnection {
    type DB = MySql;

    async fn create_table_if_not_exists<DATA: WithData>(&mut self) -> Result<(), C3p0Error> {
        let query = format!(
            r#"
                CREATE TABLE IF NOT EXISTS {} (
                    id BIGINT primary key NOT NULL AUTO_INCREMENT,
                    version int not null,
                    create_epoch_millis bigint not null,
                    update_epoch_millis bigint not null,
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
        <Record<DATA::DATA> as DbOps<MySql, DATA::DATA>>::count_all(self).await
    }

    async fn exists_by_id<DATA: WithData>(&mut self, id: u64) -> Result<bool, C3p0Error> {
        <Record<DATA::DATA> as DbOps<MySql, DATA::DATA>>::exists_by_id(self, id).await
    }

    async fn fetch_all<DATA: DataType>(&mut self) -> Result<Vec<Record<DATA>>, C3p0Error> {
        <Record<DATA> as DbOps<MySql, DATA>>::fetch_all(self).await
    }

    async fn fetch_one_optional_by_id<DATA: DataType>(
        &mut self,
        id: u64,
    ) -> Result<Option<Record<DATA>>, C3p0Error> {
        <Record<DATA> as DbOps<MySql, DATA>>::fetch_one_optional_by_id(self, id).await
    }

    async fn fetch_one_by_id<DATA: DataType>(
        &mut self,
        id: u64,
    ) -> Result<Record<DATA>, C3p0Error> {
        <Record<DATA> as DbOps<MySql, DATA>>::fetch_one_by_id(self, id).await
    }

    async fn delete<DATA: DataType>(
        &mut self,
        record: Record<DATA>,
    ) -> Result<Record<DATA>, C3p0Error> {
        <Record<DATA> as DbOps<MySql, DATA>>::delete(record, self).await
    }

    async fn delete_all<DATA: WithData>(&mut self) -> Result<u64, C3p0Error> {
        <Record<DATA::DATA> as DbOps<MySql, DATA::DATA>>::delete_all(self).await
    }

    async fn delete_by_id<DATA: WithData>(&mut self, id: u64) -> Result<u64, C3p0Error> {
        <Record<DATA::DATA> as DbOps<MySql, DATA::DATA>>::delete_by_id(self, id).await
    }

    async fn update<DATA: DataType>(
        &mut self,
        record: Record<DATA>,
    ) -> Result<Record<DATA>, C3p0Error> {
        <Record<DATA> as DbOps<MySql, DATA>>::update(record, self).await
    }

    async fn save<DATA: DataType>(
        &mut self,
        record: NewRecord<DATA>,
    ) -> Result<Record<DATA>, C3p0Error> {
        <NewRecord<DATA> as DbSave<MySql, DATA>>::save(record, self).await
    }
}
