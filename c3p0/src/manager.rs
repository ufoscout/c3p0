use crate::error::C3p0Error;
use crate::{IdType, Model, NewModel};
use std::ops::Deref;

pub trait DbManager<DATA>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
{
    type Conn;
    type Ref: Deref<Target = Self::Conn>;

    fn create_table_if_not_exists(&self, conn: Self::Ref) -> Result<u64, C3p0Error>;

    fn drop_table_if_exists(&self, conn: Self::Ref) -> Result<u64, C3p0Error>;

    fn count_all(&self, conn: Self::Ref) -> Result<IdType, C3p0Error>;

    fn exists_by_id(&self, conn: Self::Ref, id: IdType) -> Result<bool, C3p0Error>;

    fn find_all(&self, conn: Self::Ref) -> Result<Vec<Model<DATA>>, C3p0Error>;

    fn find_by_id(&self, conn: Self::Ref, id: IdType) -> Result<Option<Model<DATA>>, C3p0Error>;

    fn delete_all(&self, conn: Self::Ref) -> Result<u64, C3p0Error>;

    fn delete_by_id(&self, conn: Self::Ref, id: IdType) -> Result<u64, C3p0Error>;

    fn delete(&self, conn: Self::Ref, obj: &Model<DATA>) -> Result<u64, C3p0Error>;

    fn save(&self, conn: Self::Ref, obj: NewModel<DATA>) -> Result<Model<DATA>, C3p0Error>;

    fn update(&self, conn: Self::Ref, obj: Model<DATA>) -> Result<Model<DATA>, C3p0Error>;
}
