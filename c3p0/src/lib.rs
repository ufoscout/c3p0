use crate::error::C3p0Error;
use crate::manager::DbManager;
use serde::Deserialize;
use serde_derive::{Deserialize, Serialize};

pub mod codec;
pub mod error;
pub mod manager;
pub mod types;

pub type IdType = i64;
pub type VersionType = i32;

#[derive(Clone, Serialize, Deserialize)]
pub struct Model<DATA>
where
    DATA: Clone + serde::ser::Serialize,
{
    pub id: IdType,
    pub version: VersionType,
    #[serde(bound(deserialize = "DATA: Deserialize<'de>"))]
    pub data: DATA,
}

impl<DATA> Model<DATA>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
{
    pub fn into_new(self) -> NewModel<DATA> {
        NewModel {
            version: 0,
            data: self.data,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct NewModel<DATA>
where
    DATA: Clone + serde::ser::Serialize,
{
    pub version: VersionType,
    #[serde(bound(deserialize = "DATA: Deserialize<'de>"))]
    pub data: DATA,
}

impl<DATA> NewModel<DATA>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
{
    pub fn new(data: DATA) -> Self {
        NewModel { version: 0, data }
    }
}

impl<'a, DATA> Into<&'a IdType> for &'a Model<DATA>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
{
    fn into(self) -> &'a IdType {
        &self.id
    }
}

pub trait C3p0<DATA, DB>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
    DB: DbManager<DATA>,
{
    fn db_manager(&self) -> &DB;

    fn create_table_if_not_exists(&self, conn: &mut DB::Conn) -> Result<u64, C3p0Error> {
        self.db_manager().create_table_if_not_exists(conn)
    }

    fn drop_table_if_exists(&self, conn: &mut DB::Conn) -> Result<u64, C3p0Error> {
        self.db_manager().drop_table_if_exists(conn)
    }

    fn count_all(&self, conn: &mut DB::Conn) -> Result<IdType, C3p0Error> {
        self.db_manager().count_all(conn)
    }

    fn exists_by_id<'a, ID: Into<&'a IdType>>(
        &'a self,
        conn: &mut DB::Conn,
        id: ID,
    ) -> Result<bool, C3p0Error> {
        self.db_manager().exists_by_id(conn, *id.into())
    }

    fn find_all(&self, conn: &mut DB::Conn) -> Result<Vec<Model<DATA>>, C3p0Error> {
        self.db_manager().find_all(conn)
    }

    fn find_by_id<'a, ID: Into<&'a IdType>>(
        &'a self,
        conn: &mut DB::Conn,
        id: ID,
    ) -> Result<Option<Model<DATA>>, C3p0Error> {
        self.db_manager().find_by_id(conn, *id.into())
    }

    fn delete_all(&self, conn: &mut DB::Conn) -> Result<u64, C3p0Error> {
        self.db_manager().delete_all(conn)
    }

    fn delete_by_id<'a, ID: Into<&'a IdType>>(
        &'a self,
        conn: &mut DB::Conn,
        id: ID,
    ) -> Result<u64, C3p0Error> {
        self.db_manager().delete_by_id(conn, *id.into())
    }

    fn save(&self, conn: &mut DB::Conn, obj: NewModel<DATA>) -> Result<Model<DATA>, C3p0Error> {
        self.db_manager().save(conn, obj)
    }
}

#[derive(Clone)]
pub struct C3p0Repository<DATA, DB>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
    DB: DbManager<DATA>,
{
    db: DB,
    phantom_data: std::marker::PhantomData<DATA>,
}

impl<DATA, DB> C3p0Repository<DATA, DB>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
    DB: DbManager<DATA>,
{
    pub fn build(db: DB) -> Self {
        C3p0Repository {
            db,
            phantom_data: std::marker::PhantomData,
        }
    }
}

impl<DATA, DB> C3p0<DATA, DB> for C3p0Repository<DATA, DB>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
    DB: DbManager<DATA>,
{
    fn db_manager(&self) -> &DB {
        &self.db
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use serde_derive::{Deserialize, Serialize};
    use serde_json;

    #[test]
    fn model_should_be_serializable() -> Result<(), Box<std::error::Error>> {
        let model = Model {
            id: 1,
            version: 1,
            data: SimpleData {
                name: "test".to_owned(),
            },
        };

        let serialize = serde_json::to_string(&model)?;
        let deserialize: Model<SimpleData> = serde_json::from_str(&serialize)?;

        assert_eq!(model.id, deserialize.id);
        assert_eq!(model.version, deserialize.version);
        assert_eq!(model.data, deserialize.data);

        Ok(())
    }

    #[test]
    fn new_model_should_be_serializable() -> Result<(), Box<std::error::Error>> {
        let model = NewModel::new(SimpleData {
            name: "test".to_owned(),
        });

        let serialize = serde_json::to_string(&model)?;
        let deserialize: NewModel<SimpleData> = serde_json::from_str(&serialize)?;

        assert_eq!(model.version, deserialize.version);
        assert_eq!(model.data, deserialize.data);
        Ok(())
    }

    #[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
    struct SimpleData {
        name: String,
    }
}
