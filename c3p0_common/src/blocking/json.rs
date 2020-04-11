use crate::error::C3p0Error;
use crate::json::codec::JsonCodec;
use crate::json::model::{IdType, Model, NewModel};
use crate::sql::ForUpdate;

pub trait C3p0Json<Data, Codec>: Clone
where
    Data: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send,
    Codec: JsonCodec<Data>,
{
    type Conn;

    fn codec(&self) -> &Codec;

    fn create_table_if_not_exists(&self, conn: &mut Self::Conn) -> Result<(), C3p0Error>;

    fn drop_table_if_exists(&self, conn: &mut Self::Conn, cascade: bool) -> Result<(), C3p0Error>;

    fn count_all(&self, conn: &mut Self::Conn) -> Result<u64, C3p0Error>;

    fn exists_by_id<'a, ID: Into<&'a IdType>>(
        &'a self,
        conn: &mut Self::Conn,
        id: ID,
    ) -> Result<bool, C3p0Error>;

    fn fetch_all(&self, conn: &mut Self::Conn) -> Result<Vec<Model<Data>>, C3p0Error>;

    fn fetch_all_for_update(
        &self,
        conn: &mut Self::Conn,
        for_update: &ForUpdate,
    ) -> Result<Vec<Model<Data>>, C3p0Error>;

    fn fetch_one_optional_by_id<'a, ID: Into<&'a IdType>>(
        &'a self,
        conn: &mut Self::Conn,
        id: ID,
    ) -> Result<Option<Model<Data>>, C3p0Error>;

    fn fetch_one_optional_by_id_for_update<'a, ID: Into<&'a IdType>>(
        &'a self,
        conn: &mut Self::Conn,
        id: ID,
        for_update: &ForUpdate,
    ) -> Result<Option<Model<Data>>, C3p0Error>;

    fn fetch_one_by_id<'a, ID: Into<&'a IdType>>(
        &'a self,
        conn: &mut Self::Conn,
        id: ID,
    ) -> Result<Model<Data>, C3p0Error> {
        self.fetch_one_optional_by_id(conn, id)
            .and_then(|result| result.ok_or_else(|| C3p0Error::ResultNotFoundError))
    }

    fn fetch_one_by_id_for_update<'a, ID: Into<&'a IdType>>(
        &'a self,
        conn: &mut Self::Conn,
        id: ID,
        for_update: &ForUpdate,
    ) -> Result<Model<Data>, C3p0Error> {
        self.fetch_one_optional_by_id_for_update(conn, id, for_update)
            .and_then(|result| result.ok_or_else(|| C3p0Error::ResultNotFoundError))
    }

    fn delete(&self, conn: &mut Self::Conn, obj: Model<Data>) -> Result<Model<Data>, C3p0Error>;

    fn delete_all(&self, conn: &mut Self::Conn) -> Result<u64, C3p0Error>;

    fn delete_by_id<'a, ID: Into<&'a IdType>>(
        &'a self,
        conn: &mut Self::Conn,
        id: ID,
    ) -> Result<u64, C3p0Error>;

    fn save(&self, conn: &mut Self::Conn, obj: NewModel<Data>) -> Result<Model<Data>, C3p0Error>;

    fn update(&self, conn: &mut Self::Conn, obj: Model<Data>) -> Result<Model<Data>, C3p0Error>;
}
