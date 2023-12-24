use crate::common::executor::{
    delete, fetch_all_with_sql, fetch_one_optional_with_sql, fetch_one_with_sql,
    update, ResultWithRowCount,
};
use crate::common::to_model;
use crate::error::into_c3p0_error;
use crate::mysql::queries::build_mysql_queries;
use crate::mysql::{Db, DbRow, MySqlTx, SqlxMySqlC3p0Pool};
use async_trait::async_trait;
use c3p0_common::json::Queries;
use c3p0_common::time::utils::get_current_epoch_millis;
use c3p0_common::*;
use sqlx::mysql::MySqlQueryResult;
use sqlx::query::Query;
use sqlx::{IntoArguments, Row};

impl ResultWithRowCount for MySqlQueryResult {
    fn rows_affected(&self) -> u64 {
        self.rows_affected()
    }
}

pub trait SqlxMySqlC3p0JsonBuilder {
    fn build<DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send + Sync>(
        self,
    ) -> SqlxMySqlC3p0Json<DATA, DefaultJsonCodec>;
    fn build_with_codec<
        DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send + Sync,
        CODEC: JsonCodec<DATA>,
    >(
        self,
        codec: CODEC,
    ) -> SqlxMySqlC3p0Json<DATA, CODEC>;
}

#[derive(Clone)]
pub struct SqlxMySqlC3p0Json<DATA, CODEC: JsonCodec<DATA>>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send + Sync,
{
    phantom_data: std::marker::PhantomData<DATA>,

    codec: CODEC,
    queries: Queries,
}

impl SqlxMySqlC3p0JsonBuilder for C3p0JsonBuilder<SqlxMySqlC3p0Pool> {
    fn build<DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send + Sync>(
        self,
    ) -> SqlxMySqlC3p0Json<DATA, DefaultJsonCodec> {
        self.build_with_codec(DefaultJsonCodec {})
    }

    fn build_with_codec<
        DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send + Sync,
        CODEC: JsonCodec<DATA>,
    >(
        self,
        codec: CODEC,
    ) -> SqlxMySqlC3p0Json<DATA, CODEC> {
        SqlxMySqlC3p0Json {
            phantom_data: std::marker::PhantomData,
            codec,
            queries: build_mysql_queries(self),
        }
    }
}

impl<DATA, CODEC: JsonCodec<DATA>> SqlxMySqlC3p0Json<DATA, CODEC>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send + Sync,
{
    pub fn queries(&self) -> &Queries {
        &self.queries
    }

    #[inline]
    pub fn to_model(&self, row: &DbRow) -> Result<Model<DATA>, C3p0Error> {
        to_model(&self.codec, row, 0, 1, 2, 3, 4)
    }

    /// Allows the execution of a custom sql query and returns the first entry in the result set.
    /// For this to work, the sql query:
    /// - must be a SELECT
    /// - must declare the ID, VERSION and DATA fields in this exact order
    pub async fn fetch_one_optional_with_sql<'a, A: 'a + Send + IntoArguments<'a, Db>>(
        &self,
        tx: &mut MySqlTx,
        sql: Query<'a, Db, A>,
    ) -> Result<Option<Model<DATA>>, C3p0Error> {
        fetch_one_optional_with_sql(sql, tx.conn(), self.codec()).await
    }

    /// Allows the execution of a custom sql query and returns the first entry in the result set.
    /// For this to work, the sql query:
    /// - must be a SELECT
    /// - must declare the ID, VERSION and DATA fields in this exact order
    pub async fn fetch_one_with_sql<'a, A: 'a + Send + IntoArguments<'a, Db>>(
        &self,
        tx: &mut MySqlTx,
        sql: Query<'a, Db, A>,
    ) -> Result<Model<DATA>, C3p0Error> {
        fetch_one_with_sql(sql, tx.conn(), self.codec()).await
    }

    /// Allows the execution of a custom sql query and returns all the entries in the result set.
    /// For this to work, the sql query:
    /// - must be a SELECT
    /// - must declare the ID, VERSION and DATA fields in this exact order
    pub async fn fetch_all_with_sql<'a, A: 'a + Send + IntoArguments<'a, Db>>(
        &self,
        tx: &mut MySqlTx,
        sql: Query<'a, Db, A>,
    ) -> Result<Vec<Model<DATA>>, C3p0Error> {
        fetch_all_with_sql(sql, tx.conn(), self.codec()).await
    }
}

#[async_trait]
impl<DATA, CODEC: JsonCodec<DATA>> C3p0Json<DATA, CODEC> for SqlxMySqlC3p0Json<DATA, CODEC>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send + Sync,
{
    type Tx = MySqlTx;

    fn codec(&self) -> &CODEC {
        &self.codec
    }

    async fn create_table_if_not_exists(&self, tx: &mut Self::Tx) -> Result<(), C3p0Error> {
        sqlx::query(&self.queries.create_table_sql_query)
        .execute(tx.conn())
        .await
        .map_err(into_c3p0_error)
        .map(|_| ())
    }

    async fn drop_table_if_exists(
        &self,
        tx: &mut Self::Tx,
        cascade: bool,
    ) -> Result<(), C3p0Error> {
        let query = if cascade {
            &self.queries.drop_table_sql_query_cascade
        } else {
            &self.queries.drop_table_sql_query
        };
        sqlx::query(&query)
        .execute(tx.conn())
        .await
        .map_err(into_c3p0_error)
        .map(|_| ())
    }

    async fn count_all(&self, tx: &mut Self::Tx) -> Result<u64, C3p0Error> {
        sqlx::query(&self.queries.count_all_sql_query)
            .fetch_one(tx.conn())
            .await
            .and_then(|row| row.try_get(0))
            .map_err(into_c3p0_error)
            .map(|val: i64| val as u64)
    }

    async fn exists_by_id<'a, ID: Into<&'a IdType> + Send>(
        &'a self,
        tx: &mut Self::Tx,
        id: ID,
    ) -> Result<bool, C3p0Error> {
        sqlx::query(&self.queries.exists_by_id_sql_query)
            .bind(id.into())
            .fetch_one(tx.conn())
            .await
            .and_then(|row| row.try_get(0))
            .map_err(into_c3p0_error)
    }

    async fn fetch_all(&self, tx: &mut Self::Tx) -> Result<Vec<Model<DATA>>, C3p0Error> {
        self.fetch_all_with_sql(tx, sqlx::query(&self.queries.find_all_sql_query))
            .await
    }

    async fn fetch_one_optional_by_id<'a, ID: Into<&'a IdType> + Send>(
        &'a self,
        tx: &mut Self::Tx,
        id: ID,
    ) -> Result<Option<Model<DATA>>, C3p0Error> {
        self.fetch_one_optional_with_sql(
            tx,
            sqlx::query(&self.queries.find_by_id_sql_query).bind(id.into()),
        )
        .await
    }

    async fn fetch_one_by_id<'a, ID: Into<&'a IdType> + Send>(
        &'a self,
        tx: &mut Self::Tx,
        id: ID,
    ) -> Result<Model<DATA>, C3p0Error> {
        self.fetch_one_with_sql(
            tx,
            sqlx::query(&self.queries.find_by_id_sql_query).bind(id.into()),
        )
        .await
    }

    async fn delete(&self, tx: &mut Self::Tx, obj: Model<DATA>) -> Result<Model<DATA>, C3p0Error> {
        delete(obj, tx.conn(), &self.queries).await
    }

    async fn delete_all(&self, tx: &mut Self::Tx) -> Result<u64, C3p0Error> {
        sqlx::query(&self.queries.delete_all_sql_query)
            .execute(tx.conn())
            .await
            .map_err(into_c3p0_error)
            .map(|done| done.rows_affected())
    }

    async fn delete_by_id<'a, ID: Into<&'a IdType> + Send>(
        &'a self,
        tx: &mut Self::Tx,
        id: ID,
    ) -> Result<u64, C3p0Error> {
        sqlx::query(&self.queries.delete_by_id_sql_query)
            .bind(id.into())
            .execute(tx.conn())
            .await
            .map_err(into_c3p0_error)
            .map(|done| done.rows_affected())
    }

    async fn save(&self, tx: &mut Self::Tx, obj: NewModel<DATA>) -> Result<Model<DATA>, C3p0Error> {
        let json_data = self.codec().data_to_value(&obj.data)?;
        let create_epoch_millis = get_current_epoch_millis();
        let id = sqlx::query(&self.queries.save_sql_query)
            .bind(obj.version)
            .bind(create_epoch_millis)
            .bind(create_epoch_millis)
            .bind(&json_data)
            .execute(tx.conn())
            .await
            .map(|done| done.last_insert_id())
            .map_err(into_c3p0_error)?;

        Ok(Model {
            id: id as IdType,
            version: obj.version,
            data: obj.data,
            create_epoch_millis,
            update_epoch_millis: create_epoch_millis,
        })
    }

    async fn update(&self, tx: &mut Self::Tx, obj: Model<DATA>) -> Result<Model<DATA>, C3p0Error> {
        update(obj, tx.conn(), &self.queries, self.codec()).await
    }
}
