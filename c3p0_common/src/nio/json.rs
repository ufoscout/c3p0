use crate::error::C3p0Error;
use crate::json::codec::JsonCodec;
use crate::json::model::{IdType, Model, NewModel};
use crate::nio::pool::SqlConnectionAsync;
use crate::sql::ForUpdate;
use async_trait::async_trait;

#[async_trait]
pub trait C3p0JsonAsync<DATA, CODEC>: Clone + Send + Sync
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send,
    CODEC: JsonCodec<DATA>,
{
    type CONN: SqlConnectionAsync;

    fn codec(&self) -> &CODEC;

    async fn create_table_if_not_exists(&self, conn: &mut Self::CONN) -> Result<(), C3p0Error>;

    async fn drop_table_if_exists(
        &self,
        conn: &mut Self::CONN,
        cascade: bool,
    ) -> Result<(), C3p0Error>;

    async fn count_all(&self, conn: &mut Self::CONN) -> Result<u64, C3p0Error>;

    async fn exists_by_id<'a, ID: Into<&'a IdType> + Send>(
        &'a self,
        conn: &mut Self::CONN,
        id: ID,
    ) -> Result<bool, C3p0Error>;

    async fn fetch_all(&self, conn: &mut Self::CONN) -> Result<Vec<Model<DATA>>, C3p0Error>;

    async fn fetch_all_for_update(
        &self,
        conn: &mut Self::CONN,
        for_update: &ForUpdate,
    ) -> Result<Vec<Model<DATA>>, C3p0Error>;

    async fn fetch_one_optional_by_id<'a, ID: Into<&'a IdType> + Send>(
        &'a self,
        conn: &mut Self::CONN,
        id: ID,
    ) -> Result<Option<Model<DATA>>, C3p0Error>;

    async fn fetch_one_optional_by_id_for_update<'a, ID: Into<&'a IdType> + Send>(
        &'a self,
        conn: &mut Self::CONN,
        id: ID,
        for_update: &ForUpdate,
    ) -> Result<Option<Model<DATA>>, C3p0Error>;

    async fn fetch_one_by_id<'a, ID: Into<&'a IdType> + Send>(
        &'a self,
        conn: &mut Self::CONN,
        id: ID,
    ) -> Result<Model<DATA>, C3p0Error>;

    async fn fetch_one_by_id_for_update<'a, ID: Into<&'a IdType> + Send>(
        &'a self,
        conn: &mut Self::CONN,
        id: ID,
        for_update: &ForUpdate,
    ) -> Result<Model<DATA>, C3p0Error>;

    async fn delete(
        &self,
        conn: &mut Self::CONN,
        obj: Model<DATA>,
    ) -> Result<Model<DATA>, C3p0Error>;

    async fn delete_all(&self, conn: &mut Self::CONN) -> Result<u64, C3p0Error>;

    async fn delete_by_id<'a, ID: Into<&'a IdType> + Send>(
        &'a self,
        conn: &mut Self::CONN,
        id: ID,
    ) -> Result<u64, C3p0Error>;

    async fn save(
        &self,
        conn: &mut Self::CONN,
        obj: NewModel<DATA>,
    ) -> Result<Model<DATA>, C3p0Error>;

    async fn update(
        &self,
        conn: &mut Self::CONN,
        obj: Model<DATA>,
    ) -> Result<Model<DATA>, C3p0Error>;
}
