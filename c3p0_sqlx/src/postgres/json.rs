use crate::common::executor::{
    batch_execute, delete, fetch_all_with_sql, fetch_one_optional_with_sql, fetch_one_with_sql,
    update, ResultWithRowCount,
};
use crate::common::to_model;
use crate::error::into_c3p0_error;
use crate::postgres::queries::build_pg_queries;
use crate::postgres::{Db, DbRow, SqlxPgC3p0Pool, SqlxPgConnection};
use async_trait::async_trait;
use c3p0_common::json::Queries;
use c3p0_common::*;
use sqlx::postgres::PgQueryResult;
use sqlx::query::Query;
use sqlx::{IntoArguments, Row};

impl ResultWithRowCount for PgQueryResult {
    fn rows_affected(&self) -> u64 {
        self.rows_affected()
    }
}

pub trait SqlxPgC3p0JsonBuilder {
    fn build<DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send + Sync>(
        self,
    ) -> SqlxPgC3p0Json<DATA, DefaultJsonCodec>;
    fn build_with_codec<
        DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send + Sync,
        CODEC: JsonCodec<DATA>,
    >(
        self,
        codec: CODEC,
    ) -> SqlxPgC3p0Json<DATA, CODEC>;
}

#[derive(Clone)]
pub struct SqlxPgC3p0Json<DATA, CODEC: JsonCodec<DATA>>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send + Sync,
{
    phantom_data: std::marker::PhantomData<DATA>,

    codec: CODEC,
    queries: Queries,
}

impl SqlxPgC3p0JsonBuilder for C3p0JsonBuilder<SqlxPgC3p0Pool> {
    fn build<DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send + Sync>(
        self,
    ) -> SqlxPgC3p0Json<DATA, DefaultJsonCodec> {
        self.build_with_codec(DefaultJsonCodec {})
    }

    fn build_with_codec<
        DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send + Sync,
        CODEC: JsonCodec<DATA>,
    >(
        self,
        codec: CODEC,
    ) -> SqlxPgC3p0Json<DATA, CODEC> {
        SqlxPgC3p0Json {
            phantom_data: std::marker::PhantomData,
            codec,
            queries: build_pg_queries(self),
        }
    }
}

impl<DATA, CODEC: JsonCodec<DATA>> SqlxPgC3p0Json<DATA, CODEC>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send + Sync,
{
    pub fn queries(&self) -> &Queries {
        &self.queries
    }

    #[inline]
    pub fn to_model(&self, row: &DbRow) -> Result<Model<DATA>, C3p0Error> {
        to_model(&self.codec, row, 0, 1, 2)
    }

    /// Allows the execution of a custom sql query and returns the first entry in the result set.
    /// For this to work, the sql query:
    /// - must be a SELECT
    /// - must declare the ID, VERSION and DATA fields in this exact order
    pub async fn fetch_one_optional_with_sql<'a, A: 'a + Send + IntoArguments<'a, Db>>(
        &self,
        conn: &mut SqlxPgConnection,
        sql: Query<'a, Db, A>,
    ) -> Result<Option<Model<DATA>>, C3p0Error> {
        fetch_one_optional_with_sql(sql, conn.get_conn(), self.codec()).await
    }

    /// Allows the execution of a custom sql query and returns the first entry in the result set.
    /// For this to work, the sql query:
    /// - must be a SELECT
    /// - must declare the ID, VERSION and DATA fields in this exact order
    pub async fn fetch_one_with_sql<'a, A: 'a + Send + IntoArguments<'a, Db>>(
        &self,
        conn: &mut SqlxPgConnection,
        sql: Query<'a, Db, A>,
    ) -> Result<Model<DATA>, C3p0Error> {
        fetch_one_with_sql(sql, conn.get_conn(), self.codec()).await
    }

    /// Allows the execution of a custom sql query and returns all the entries in the result set.
    /// For this to work, the sql query:
    /// - must be a SELECT
    /// - must declare the ID, VERSION and DATA fields in this exact order
    pub async fn fetch_all_with_sql<'a, A: 'a + Send + IntoArguments<'a, Db>>(
        &self,
        conn: &mut SqlxPgConnection,
        sql: Query<'a, Db, A>,
    ) -> Result<Vec<Model<DATA>>, C3p0Error> {
        fetch_all_with_sql(sql, conn.get_conn(), self.codec()).await
    }
}

#[async_trait]
impl<DATA, CODEC: JsonCodec<DATA>> C3p0Json<DATA, CODEC> for SqlxPgC3p0Json<DATA, CODEC>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send + Sync,
{
    type Conn = SqlxPgConnection;

    fn codec(&self) -> &CODEC {
        &self.codec
    }

    async fn create_table_if_not_exists(&self, conn: &mut Self::Conn) -> Result<(), C3p0Error> {
        batch_execute(&self.queries.create_table_sql_query, conn.get_conn()).await
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
        batch_execute(query, conn.get_conn()).await
    }

    async fn count_all(&self, conn: &mut Self::Conn) -> Result<u64, C3p0Error> {
        sqlx::query(&self.queries.count_all_sql_query)
            .fetch_one(conn.get_conn())
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
            .fetch_one(conn.get_conn())
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
        delete(obj, conn.get_conn(), &self.queries).await
    }

    async fn delete_all(&self, conn: &mut Self::Conn) -> Result<u64, C3p0Error> {
        sqlx::query(&self.queries.delete_all_sql_query)
            .execute(conn.get_conn())
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
            .execute(conn.get_conn())
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
            .fetch_one(conn.get_conn())
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
        update(obj, conn.get_conn(), &self.queries, self.codec()).await
    }
}
