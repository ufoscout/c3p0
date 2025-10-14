use sqlx::{MySql, MySqlConnection};

use crate::{C3p0Error, DataType, DbOps, DbSave, NewRecord, Record, Tx, error::into_c3p0_error};

impl Tx for MySqlConnection {

    type DB = MySql;

    async fn create_table_if_not_exists<DATA: DataType>(&mut self) -> Result<(), C3p0Error> {
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
            DATA::TABLE_NAME,
        );

        sqlx::query(&query)
            .execute(self)
            .await
            .map_err(into_c3p0_error)
            .map(|_| ())
    }

    async fn drop_table_if_exists<DATA: DataType>(&mut self, cascade: bool) -> Result<(), C3p0Error> {
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

    async fn fetch_all_with_sql<'a, DATA: DataType, A: 'a + Send + sqlx::IntoArguments<'a, MySql>>(
        &mut self,
        sql: sqlx::query::Query<'a, MySql, A>,
    ) -> Result<Vec<Record<DATA>>, C3p0Error> {
        <Record<DATA> as DbOps<MySql, DATA>>::fetch_all_with_sql(self, sql).await
    }

    async fn fetch_one_optional_with_sql<
        'a,
        DATA: DataType,
        A: 'a + Send + sqlx::IntoArguments<'a, MySql>,
    >(
        &mut self,
        sql: sqlx::query::Query<'a, MySql, A>,
    ) -> Result<Option<Record<DATA>>, C3p0Error> {
        <Record<DATA> as DbOps<MySql, DATA>>::fetch_one_optional_with_sql(self, sql).await
    }

    async fn fetch_one_with_sql<'a, DATA: DataType, A: 'a + Send + sqlx::IntoArguments<'a, MySql>>(
        &mut self,
        sql: sqlx::query::Query<'a, MySql, A>,
    ) -> Result<Record<DATA>, C3p0Error> {
        <Record<DATA> as DbOps<MySql, DATA>>::fetch_one_with_sql(self, sql).await
    }

    async fn count_all<DATA: DataType>(&mut self) -> Result<u64, C3p0Error> {
        <Record<DATA> as DbOps<MySql, DATA>>::count_all(self).await
    }

    async fn exists_by_id<DATA: DataType>(&mut self, id: u64) -> Result<bool, C3p0Error> {
        <Record<DATA> as DbOps<MySql, DATA>>::exists_by_id(self, id).await
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

    async fn fetch_one_by_id<DATA: DataType>(&mut self, id: u64) -> Result<Record<DATA>, C3p0Error> {
        <Record<DATA> as DbOps<MySql, DATA>>::fetch_one_by_id(self, id).await
    }

    async fn delete<DATA: DataType>(
        &mut self,
        record: Record<DATA>,
    ) -> Result<Record<DATA>, C3p0Error> {
        <Record<DATA> as DbOps<MySql, DATA>>::delete(record, self).await
    }

    async fn delete_all<DATA: DataType>(&mut self) -> Result<u64, C3p0Error> {
        <Record<DATA> as DbOps<MySql, DATA>>::delete_all(self).await
    }

    async fn delete_by_id<DATA: DataType>(&mut self, id: u64) -> Result<u64, C3p0Error> {
        <Record<DATA> as DbOps<MySql, DATA>>::delete_by_id(self, id).await
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
