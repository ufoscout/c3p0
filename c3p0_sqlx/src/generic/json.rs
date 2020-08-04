use crate::error::into_c3p0_error;
use async_trait::async_trait;
use c3p0_common::json::Queries;
use c3p0_common::*;
use sqlx::query::Query;
use sqlx::{Done, Database};
use sqlx::{IntoArguments, Row};
use std::iter::Iterator;
use crate::common::to_model;
use crate::generic::{SqlxC3p0Pool, SqlxConnection};

pub trait SqlxC3p0JsonBuilder<DB>
    where DB: Clone + Database + Sync, <DB as Database>::Connection : Sync,
          for<'c> <DB as sqlx::database::HasArguments<'c>>::Arguments: sqlx::IntoArguments<'c, DB>,
          for<'c> &'c sqlx::Transaction<'c, DB> : sqlx::Executor<'c, Database = DB>,
          for<'c> i32: sqlx::types::Type<DB> + sqlx::decode::Decode<'c, DB>,
          for<'c> i64: sqlx::types::Type<DB> + sqlx::decode::Decode<'c, DB>,
          for<'c> serde_json::value::Value: sqlx::types::Type<DB> + sqlx::decode::Decode<'c, DB>,
{

    fn build<DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send + Sync>(
        self,
    ) -> SqlxC3p0Json<DB, DATA, DefaultJsonCodec>;
    fn build_with_codec<
        DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send + Sync,
        CODEC: JsonCodec<DATA>,
    >(
        self,
        codec: CODEC,
    ) -> SqlxC3p0Json<DB, DATA, CODEC>;
}

#[derive(Clone)]
pub struct SqlxC3p0Json<DB, DATA, CODEC: JsonCodec<DATA>>
    where DB: Clone + Database + Sync, <DB as Database>::Connection : Sync,
          for<'c> <DB as sqlx::database::HasArguments<'c>>::Arguments: sqlx::IntoArguments<'c, DB>,
          for<'c> &'c sqlx::Transaction<'c, DB> : sqlx::Executor<'c, Database = DB>,
          for<'c> i32: sqlx::types::Type<DB> + sqlx::decode::Decode<'c, DB>,
          for<'c> i64: sqlx::types::Type<DB> + sqlx::decode::Decode<'c, DB>,
          for<'c> serde_json::value::Value: sqlx::types::Type<DB> + sqlx::decode::Decode<'c, DB>,
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send + Sync,
{
    phantom_data: std::marker::PhantomData<DATA>,
    phantom_db: std::marker::PhantomData<DB>,

    codec: CODEC,
    queries: Queries,
}

impl <DB> SqlxC3p0JsonBuilder<DB> for C3p0JsonBuilder<SqlxC3p0Pool<DB>>
    where DB: Clone + Database + Sync, <DB as Database>::Connection : Sync,
          for<'c> <DB as sqlx::database::HasArguments<'c>>::Arguments: sqlx::IntoArguments<'c, DB>,
          for<'c> &'c sqlx::Transaction<'c, DB> : sqlx::Executor<'c, Database = DB>,
          for<'c> i32: sqlx::types::Type<DB> + sqlx::decode::Decode<'c, DB>,
          for<'c> i64: sqlx::types::Type<DB> + sqlx::decode::Decode<'c, DB>,
          for<'c> serde_json::value::Value: sqlx::types::Type<DB> + sqlx::decode::Decode<'c, DB>,
{
    fn build<DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send + Sync>(
        self,
    ) -> SqlxC3p0Json<DB, DATA, DefaultJsonCodec> {
        self.build_with_codec(DefaultJsonCodec {})
    }

    fn build_with_codec<
        DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send + Sync,
        CODEC: JsonCodec<DATA>,
    >(
        self,
        codec: CODEC,
    ) -> SqlxC3p0Json<DB, DATA, CODEC> {
        unimplemented!()
        // SqlxC3p0Json {
        //     phantom_data: std::marker::PhantomData,
        //     codec,
        //     queries: build_pg_queries(self),
        // }
    }
}

impl<DB, DATA, CODEC: JsonCodec<DATA>> SqlxC3p0Json<DB, DATA, CODEC>
where
    DB: Clone + Database + Sync, <DB as Database>::Connection : Sync,
    for<'c> <DB as sqlx::database::HasArguments<'c>>::Arguments: sqlx::IntoArguments<'c, DB>,
    for<'c> &'c sqlx::Transaction<'c, DB> : sqlx::Executor<'c, Database = DB>,
    for<'c> i32: sqlx::types::Type<DB> + sqlx::decode::Decode<'c, DB>,
    for<'c> i64: sqlx::types::Type<DB> + sqlx::decode::Decode<'c, DB>,
    for<'c> serde_json::value::Value: sqlx::types::Type<DB> + sqlx::decode::Decode<'c, DB>,
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send + Sync,
{
    pub fn queries(&self) -> &Queries {
        &self.queries
    }

    #[inline]
    pub fn to_model<R: Row<Database = DB>>(&self, row: &R) -> Result<Model<DATA>, C3p0Error> {
        to_model(&self.codec, row, 0, 1, 2)
    }

    /// Allows the execution of a custom sql query and returns the first entry in the result set.
    /// For this to work, the sql query:
    /// - must be a SELECT
    /// - must declare the ID, VERSION and DATA fields in this exact order
    pub async fn fetch_one_optional_with_sql<'a, A: 'a + Send + IntoArguments<'a, DB>>(
        &self,
        conn: &mut SqlxConnection<DB>,
        sql: Query<'a, DB, A>,
    ) -> Result<Option<Model<DATA>>, C3p0Error> {
        sql.fetch_optional(conn.as_executor())
            .await
            .map_err(into_c3p0_error)?
            .map(|row| self.to_model(&row))
            .transpose()
    }

    /// Allows the execution of a custom sql query and returns the first entry in the result set.
    /// For this to work, the sql query:
    /// - must be a SELECT
    /// - must declare the ID, VERSION and DATA fields in this exact order
    pub async fn fetch_one_with_sql<'a, A: 'a + Send + IntoArguments<'a, DB>>(
        &self,
        conn: &mut SqlxConnection<DB>,
        sql: Query<'a, DB, A>,
    ) -> Result<Model<DATA>, C3p0Error> {
        sql.fetch_one(conn.as_executor())
            .await
            .map_err(into_c3p0_error)
            .and_then(|row| self.to_model(&row))
    }

    /// Allows the execution of a custom sql query and returns all the entries in the result set.
    /// For this to work, the sql query:
    /// - must be a SELECT
    /// - must declare the ID, VERSION and DATA fields in this exact order
    pub async fn fetch_all_with_sql<'a, A: 'a + Send + IntoArguments<'a, DB>>(
        &self,
        conn: &mut SqlxConnection<DB>,
        sql: Query<'a, DB, A>,
    ) -> Result<Vec<Model<DATA>>, C3p0Error> {
        sql.fetch_all(conn.as_executor())
            .await
            .map_err(into_c3p0_error)?
            .iter()
            .map(|row| self.to_model(&row))
            .collect::<Result<Vec<_>, C3p0Error>>()
    }
}

#[async_trait]
impl<DB, DATA, CODEC: JsonCodec<DATA>> C3p0Json<DATA, CODEC> for SqlxC3p0Json<DB, DATA, CODEC>
where
    DB: Clone + Database + Sync, <DB as Database>::Connection : Sync,
    for<'c> <DB as sqlx::database::HasArguments<'c>>::Arguments: sqlx::IntoArguments<'c, DB>,
    for<'c> &'c sqlx::Transaction<'c, DB> : sqlx::Executor<'c, Database = DB>,
    for<'c> i32: sqlx::types::Type<DB> + sqlx::decode::Decode<'c, DB>,
    for<'c> i64: sqlx::types::Type<DB> + sqlx::decode::Decode<'c, DB>,
    for<'c> serde_json::value::Value: sqlx::types::Type<DB> + sqlx::decode::Decode<'c, DB>,
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send + Sync,
{
    type Conn = SqlxConnection<DB>;

    fn codec(&self) -> &CODEC {
        &self.codec
    }

    async fn create_table_if_not_exists(&self, conn: &mut Self::Conn) -> Result<(), C3p0Error> {
        sqlx::query(&self.queries.create_table_sql_query)
            .execute(conn.as_executor())
            .await
            .map_err(into_c3p0_error)
            .map(|_| ())
    }

    async fn drop_table_if_exists(
        &self,
        conn: &mut Self::Conn,
        cascade: bool,
    ) -> Result<(), C3p0Error> {
        let query = if cascade {
            &self.queries.drop_table_sql_query_cascade
        } else {
            &self.queries.drop_table_sql_query
        };
        sqlx::query(query)
            .execute(conn.as_executor())
            .await
            .map_err(into_c3p0_error)
            .map(|_| ())
    }

    async fn count_all(&self, conn: &mut Self::Conn) -> Result<u64, C3p0Error> {
        sqlx::query(&self.queries.count_all_sql_query)
            .fetch_one(conn.as_executor())
            .await
            .and_then(|row| row.try_get(0))
            .map_err(into_c3p0_error)
            .map(|val: i64| val as u64)
    }

    async fn exists_by_id<'a, ID: Into<&'a IdType> + Send>(
        &'a self,
        conn: &mut Self::Conn,
        id: ID,
    ) -> Result<bool, C3p0Error> {
        sqlx::query(&self.queries.exists_by_id_sql_query)
            .bind(id.into())
            .fetch_one(conn.as_executor())
            .await
            .and_then(|row| row.try_get(0))
            .map_err(into_c3p0_error)
    }

    async fn fetch_all(&self, conn: &mut Self::Conn) -> Result<Vec<Model<DATA>>, C3p0Error> {
        self.fetch_all_with_sql(conn, sqlx::query(&self.queries.find_all_sql_query))
            .await
    }

    async fn fetch_all_for_update(
        &self,
        conn: &mut Self::Conn,
        for_update: &ForUpdate,
    ) -> Result<Vec<Model<DATA>>, C3p0Error> {
        let sql = format!(
            "{}\n{}",
            &self.queries.find_all_sql_query,
            for_update.to_sql()
        );
        self.fetch_all_with_sql(conn, sqlx::query(&sql)).await
    }

    async fn fetch_one_optional_by_id<'a, ID: Into<&'a IdType> + Send>(
        &'a self,
        conn: &mut Self::Conn,
        id: ID,
    ) -> Result<Option<Model<DATA>>, C3p0Error> {
        self.fetch_one_optional_with_sql(
            conn,
            sqlx::query(&self.queries.find_by_id_sql_query).bind(id.into()),
        )
        .await
    }

    async fn fetch_one_optional_by_id_for_update<'a, ID: Into<&'a IdType> + Send>(
        &'a self,
        conn: &mut Self::Conn,
        id: ID,
        for_update: &ForUpdate,
    ) -> Result<Option<Model<DATA>>, C3p0Error> {
        let sql = format!(
            "{}\n{}",
            &self.queries.find_by_id_sql_query,
            for_update.to_sql()
        );
        self.fetch_one_optional_with_sql(conn, sqlx::query(&sql).bind(id.into()))
            .await
    }

    async fn fetch_one_by_id<'a, ID: Into<&'a IdType> + Send>(
        &'a self,
        conn: &mut Self::Conn,
        id: ID,
    ) -> Result<Model<DATA>, C3p0Error> {
        self.fetch_one_with_sql(
            conn,
            sqlx::query(&self.queries.find_by_id_sql_query).bind(id.into()),
        )
        .await
        // self.fetch_one_optional_by_id(conn, id)
        //     .await
        //     .and_then(|result| result.ok_or_else(|| C3p0Error::ResultNotFoundError))
    }

    async fn fetch_one_by_id_for_update<'a, ID: Into<&'a IdType> + Send>(
        &'a self,
        conn: &mut Self::Conn,
        id: ID,
        for_update: &ForUpdate,
    ) -> Result<Model<DATA>, C3p0Error> {
        let sql = format!(
            "{}\n{}",
            &self.queries.find_by_id_sql_query,
            for_update.to_sql()
        );
        self.fetch_one_with_sql(conn, sqlx::query(&sql).bind(id.into()))
            .await

        // self.fetch_one_optional_by_id_for_update(conn, id, for_update)
        //     .await
        //     .and_then(|result| result.ok_or_else(|| C3p0Error::ResultNotFoundError))
    }

    async fn delete(
        &self,
        conn: &mut Self::Conn,
        obj: Model<DATA>,
    ) -> Result<Model<DATA>, C3p0Error> {
        let result = sqlx::query(&self.queries.delete_sql_query)
            .bind(&obj.id)
            .bind(&obj.version)
            .execute(conn.as_executor())
            .await
            .map_err(into_c3p0_error)?
            .rows_affected();

        if result == 0 {
            return Err(C3p0Error::OptimisticLockError{ message: format!("Cannot update data in table [{}] with id [{}], version [{}]: data was changed!",
                                                                        &self.queries.qualified_table_name, &obj.id, &obj.version
            )});
        }

        Ok(obj)
    }

    async fn delete_all(&self, conn: &mut Self::Conn) -> Result<u64, C3p0Error> {
        sqlx::query(&self.queries.delete_all_sql_query)
            .execute(conn.as_executor())
            .await
            .map_err(into_c3p0_error)
            .map(|done| done.rows_affected())
    }

    async fn delete_by_id<'a, ID: Into<&'a IdType> + Send>(
        &'a self,
        conn: &mut Self::Conn,
        id: ID,
    ) -> Result<u64, C3p0Error> {
        sqlx::query(&self.queries.delete_by_id_sql_query)
            .bind(id.into())
            .execute(conn.as_executor())
            .await
            .map_err(into_c3p0_error)
            .map(|done| done.rows_affected())
    }

    async fn save(
        &self,
        conn: &mut Self::Conn,
        obj: NewModel<DATA>,
    ) -> Result<Model<DATA>, C3p0Error> {
        let json_data = self.codec().to_value(&obj.data)?;

        let id = sqlx::query(&self.queries.save_sql_query)
            .bind(&obj.version)
            .bind(&json_data)
            .fetch_one(conn.as_executor())
            .await
            .and_then(|row| row.try_get(0))
            .map_err(into_c3p0_error)?;

        Ok(Model {
            id,
            version: obj.version,
            data: obj.data,
        })
    }

    async fn update(
        &self,
        conn: &mut Self::Conn,
        obj: Model<DATA>,
    ) -> Result<Model<DATA>, C3p0Error> {
        let json_data = self.codec().to_value(&obj.data)?;

        let updated_model = Model {
            id: obj.id,
            version: obj.version + 1,
            data: obj.data,
        };

        let result = sqlx::query(&self.queries.update_sql_query)
            .bind(&updated_model.version)
            .bind(&json_data)
            .bind(&updated_model.id)
            .bind(&obj.version)
            .execute(conn.as_executor())
            .await
            .map_err(into_c3p0_error)
            .map(|done| done.rows_affected())?;

        if result == 0 {
            return Err(C3p0Error::OptimisticLockError{ message: format!("Cannot update data in table [{}] with id [{}], version [{}]: data was changed!",
                                                                        &self.queries.qualified_table_name, &updated_model.id, &obj.version
            )});
        }

        Ok(updated_model)
    }
}
