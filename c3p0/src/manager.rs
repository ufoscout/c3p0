use crate::error::C3p0Error;
use crate::{IdType, Model, NewModel};

pub trait DbManager<DATA>
    where DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned {
    type Conn;

    fn create_table_if_not_exists(&self, conn: &mut Self::Conn) -> Result<u64, C3p0Error>;

    fn drop_table_if_exists(&self, conn: &mut Self::Conn) -> Result<u64, C3p0Error>;

    fn count_all(&self, conn: &mut Self::Conn) -> Result<IdType, C3p0Error>;

    fn exists_by_id<'a>(
        &'a self,
        conn: &mut Self::Conn,
        id: &IdType,
    ) -> Result<bool, C3p0Error>;

    fn find_all(&self, conn: &mut Self::Conn) -> Result<Vec<Model<DATA>>, C3p0Error>;

    fn find_by_id<'a>(
        &'a self,
        conn: &mut Self::Conn,
        id: &IdType,
    ) -> Result<Option<Model<DATA>>, C3p0Error>;

    fn delete_all(&self, conn: &mut Self::Conn) -> Result<u64, C3p0Error>;

    fn delete_by_id<'a>(
        &'a self,
        conn: &mut Self::Conn,
        id: &IdType,
    ) -> Result<u64, C3p0Error>;

    fn save(&self, conn: &mut Self::Conn, obj: NewModel<DATA>) -> Result<Model<DATA>, C3p0Error>;
}
