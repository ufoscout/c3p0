use crate::common::{build_mysql_queries, to_model};
use crate::{MysqlC3p0Pool, MysqlConnection};
use async_trait::async_trait;
use c3p0_common::json::Queries;
use c3p0_common::*;
use mysql_async::prelude::ToValue;
use mysql_async::Row;

pub trait MysqlC3p0JsonBuilder {
    fn build<DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send + Sync>(
        self,
    ) -> MysqlC3p0Json<DATA, DefaultJsonCodec>;
    fn build_with_codec<
        DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send + Sync,
        CODEC: JsonCodec<DATA>,
    >(
        self,
        codec: CODEC,
    ) -> MysqlC3p0Json<DATA, CODEC>;
}

impl MysqlC3p0JsonBuilder for C3p0JsonBuilder<MysqlC3p0Pool> {
    fn build<DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send + Sync>(
        self,
    ) -> MysqlC3p0Json<DATA, DefaultJsonCodec> {
        self.build_with_codec(DefaultJsonCodec {})
    }

    fn build_with_codec<
        DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send + Sync,
        CODEC: JsonCodec<DATA>,
    >(
        self,
        codec: CODEC,
    ) -> MysqlC3p0Json<DATA, CODEC> {
        MysqlC3p0Json {
            phantom_data: std::marker::PhantomData,
            codec,
            queries: build_mysql_queries(self),
        }
    }
}

#[derive(Clone)]
pub struct MysqlC3p0Json<DATA, CODEC: JsonCodec<DATA>>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send + Sync,
{
    phantom_data: std::marker::PhantomData<DATA>,

    codec: CODEC,
    queries: Queries,
}

impl<DATA, CODEC: JsonCodec<DATA>> MysqlC3p0Json<DATA, CODEC>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send + Sync,
{
    pub fn queries(&self) -> &Queries {
        &self.queries
    }

    #[inline]
    pub fn to_model(&self, row: &Row) -> Result<Model<DATA>, Box<dyn std::error::Error>> {
        to_model(&self.codec, row, 0, 1, 2)
    }

    /// Allows the execution of a custom sql query and returns the first entry in the result set.
    /// For this to work, the sql query:
    /// - must be a SELECT
    /// - must declare the ID, VERSION and DATA fields in this exact order
    pub async fn fetch_one_optional_with_sql(
        &self,
        conn: &mut MysqlConnection,
        sql: &str,
        params: &[&(dyn ToValue)],
    ) -> Result<Option<Model<DATA>>, C3p0Error> {
        conn.fetch_one_optional(sql, params, |row| self.to_model(row))
            .await
    }

    /// Allows the execution of a custom sql query and returns the first entry in the result set.
    /// For this to work, the sql query:
    /// - must be a SELECT
    /// - must declare the ID, VERSION and DATA fields in this exact order
    pub async fn fetch_one_with_sql(
        &self,
        conn: &mut MysqlConnection,
        sql: &str,
        params: &[&(dyn ToValue)],
    ) -> Result<Model<DATA>, C3p0Error> {
        conn.fetch_one(sql, params, |row| self.to_model(row)).await
    }

    /// Allows the execution of a custom sql query and returns all the entries in the result set.
    /// For this to work, the sql query:
    /// - must be a SELECT
    /// - must declare the ID, VERSION and DATA fields in this exact order
    pub async fn fetch_all_with_sql(
        &self,
        conn: &mut MysqlConnection,
        sql: &str,
        params: &[&(dyn ToValue)],
    ) -> Result<Vec<Model<DATA>>, C3p0Error> {
        conn.fetch_all(sql, params, |row| self.to_model(row)).await
    }
}

#[async_trait]
impl<DATA, CODEC: JsonCodec<DATA>> C3p0Json<DATA, CODEC> for MysqlC3p0Json<DATA, CODEC>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send + Sync,
{
    type Conn = MysqlConnection;

    fn codec(&self) -> &CODEC {
        &self.codec
    }

    async fn create_table_if_not_exists(&self, _conn: &mut Self::Conn) -> Result<(), C3p0Error> {
        unimplemented!()
    }

    async fn drop_table_if_exists(
        &self,
        _conn: &mut Self::Conn,
        _cascade: bool,
    ) -> Result<(), C3p0Error> {
        unimplemented!()
    }

    async fn count_all(&self, _conn: &mut Self::Conn) -> Result<u64, C3p0Error> {
        unimplemented!()
    }

    async fn exists_by_id<'a, ID: Into<&'a IdType> + Send>(
        &'a self,
        _conn: &mut Self::Conn,
        _id: ID,
    ) -> Result<bool, C3p0Error> {
        unimplemented!()
    }

    async fn fetch_all(&self, _conn: &mut Self::Conn) -> Result<Vec<Model<DATA>>, C3p0Error> {
        unimplemented!()
    }

    async fn fetch_all_for_update(
        &self,
        _conn: &mut Self::Conn,
        _for_update: &ForUpdate,
    ) -> Result<Vec<Model<DATA>>, C3p0Error> {
        unimplemented!()
    }

    async fn fetch_one_optional_by_id<'a, ID: Into<&'a IdType> + Send>(
        &'a self,
        _conn: &mut Self::Conn,
        _id: ID,
    ) -> Result<Option<Model<DATA>>, C3p0Error> {
        unimplemented!()
    }

    async fn fetch_one_optional_by_id_for_update<'a, ID: Into<&'a IdType> + Send>(
        &'a self,
        _conn: &mut Self::Conn,
        _id: ID,
        _for_update: &ForUpdate,
    ) -> Result<Option<Model<DATA>>, C3p0Error> {
        unimplemented!()
    }

    async fn fetch_one_by_id<'a, ID: Into<&'a IdType> + Send>(
        &'a self,
        _conn: &mut Self::Conn,
        _id: ID,
    ) -> Result<Model<DATA>, C3p0Error> {
        unimplemented!()
    }

    async fn fetch_one_by_id_for_update<'a, ID: Into<&'a IdType> + Send>(
        &'a self,
        _conn: &mut Self::Conn,
        _id: ID,
        _for_update: &ForUpdate,
    ) -> Result<Model<DATA>, C3p0Error> {
        unimplemented!()
    }

    async fn delete(
        &self,
        _conn: &mut Self::Conn,
        _obj: Model<DATA>,
    ) -> Result<Model<DATA>, C3p0Error> {
        unimplemented!()
    }

    async fn delete_all(&self, _conn: &mut Self::Conn) -> Result<u64, C3p0Error> {
        unimplemented!()
    }

    async fn delete_by_id<'a, ID: Into<&'a IdType> + Send>(
        &'a self,
        _conn: &mut Self::Conn,
        _id: ID,
    ) -> Result<u64, C3p0Error> {
        unimplemented!()
    }

    async fn save(
        &self,
        _conn: &mut Self::Conn,
        _obj: NewModel<DATA>,
    ) -> Result<Model<DATA>, C3p0Error> {
        unimplemented!()
    }

    async fn update(
        &self,
        _conn: &mut Self::Conn,
        _obj: Model<DATA>,
    ) -> Result<Model<DATA>, C3p0Error> {
        unimplemented!()
    }
}
