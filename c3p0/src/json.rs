use crate::client::Row;
use crate::error::C3p0Error;
use crate::json::codec::JsonCodec;
use crate::pool::ConnectionBase;
use serde::Deserialize;
use serde_derive::{Deserialize, Serialize};

pub mod codec;

pub trait JsonManagerBase<DATA, CODEC: JsonCodec<DATA>>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
{
    type Conn: ConnectionBase;

    fn codec(&self) -> &CODEC;

    fn to_model(&self, row: &Row) -> Result<Model<DATA>, C3p0Error>;

    fn id_field_name(&self) -> &str;
    fn version_field_name(&self) -> &str;
    fn data_field_name(&self) -> &str;
    fn table_name(&self) -> &str;
    fn schema_name(&self) -> &Option<String>;
    fn qualified_table_name(&self) -> &str;

    fn count_all_sql_query(&self) -> &str;
    fn exists_by_id_sql_query(&self) -> &str;

    fn find_all_sql_query(&self) -> &str;
    fn find_by_id_sql_query(&self) -> &str;

    fn delete_sql_query(&self) -> &str;
    fn delete_all_sql_query(&self) -> &str;
    fn delete_by_id_sql_query(&self) -> &str;

    fn save_sql_query(&self) -> &str;

    fn update_sql_query(&self) -> &str;

    fn create_table_sql_query(&self) -> &str;
    fn drop_table_sql_query(&self) -> &str;
    fn lock_table_exclusively_sql_query(&self) -> &str;

    fn create_table_if_not_exists(&self, conn: &Self::Conn) -> Result<(), C3p0Error> {
        conn.execute(self.create_table_sql_query(), &[])?;
        Ok(())
    }

    fn drop_table_if_exists(&self, conn: &Self::Conn) -> Result<(), C3p0Error> {
        conn.execute(self.drop_table_sql_query(), &[])?;
        Ok(())
    }

    fn lock_table_exclusively(&self, conn: &Self::Conn) -> Result<(), C3p0Error> {
        conn.batch_execute(self.lock_table_exclusively_sql_query())?;
        Ok(())
    }

    fn count_all(&self, conn: &Self::Conn) -> Result<i64, C3p0Error> {
        conn.fetch_one_value(self.count_all_sql_query(), &[])
    }

    fn exists_by_id(&self, conn: &Self::Conn, id: i64) -> Result<bool, C3p0Error> {
        conn.fetch_one_value(self.exists_by_id_sql_query(), &[&id])
    }

    fn find_all(&self, conn: &Self::Conn) -> Result<Vec<Model<DATA>>, C3p0Error> {
        conn.fetch_all(
            self.find_all_sql_query(),
            &[],
            |row| Ok(self.to_model(row)?),
        )
    }

    fn find_by_id(&self, conn: &Self::Conn, id: i64) -> Result<Option<Model<DATA>>, C3p0Error> {
        conn.fetch_one_option(self.find_by_id_sql_query(), &[&id], |row| {
            Ok(self.to_model(row)?)
        })
    }

    fn delete(&self, conn: &Self::Conn, obj: &Model<DATA>) -> Result<u64, C3p0Error> {
        let result = conn.execute(self.delete_sql_query(), &[&obj.id, &obj.version])?;

        if result == 0 {
            return Err(C3p0Error::OptimisticLockError{ message: format!("Cannot update data in table [{}] with id [{}], version [{}]: data was changed!",
                                                                        self.qualified_table_name(), &obj.id, &obj.version
            )});
        }

        Ok(result)
    }

    fn delete_all(&self, conn: &Self::Conn) -> Result<u64, C3p0Error> {
        conn.execute(self.delete_all_sql_query(), &[])
    }

    fn delete_by_id(&self, conn: &Self::Conn, id: i64) -> Result<u64, C3p0Error> {
        conn.execute(self.delete_by_id_sql_query(), &[&id])
    }

    fn update(&self, conn: &Self::Conn, obj: Model<DATA>) -> Result<Model<DATA>, C3p0Error> {
        let json_data = self.codec().to_value(&obj.data)?;

        let updated_model = Model {
            id: obj.id,
            version: obj.version + 1,
            data: obj.data,
        };

        let result = conn.execute(
            self.update_sql_query(),
            &[
                &updated_model.version,
                &json_data,
                &updated_model.id,
                &obj.version,
            ],
        )?;

        if result == 0 {
            return Err(C3p0Error::OptimisticLockError{ message: format!("Cannot update data in table [{}] with id [{}], version [{}]: data was changed!",
                                                                        self.qualified_table_name(), &updated_model.id, &obj.version
            )});
        }

        Ok(updated_model)
    }

    fn save<M: Into<NewModel<DATA>>>(&self, conn: &Self::Conn, data: M) -> Result<Model<DATA>, C3p0Error> {
        let obj = data.into();
        let json_data = self.codec().to_value(&obj.data)?;
        let id = conn.fetch_one_value(self.save_sql_query(), &[&obj.version, &json_data])?;
        Ok(Model {
            id,
            version: obj.version,
            data: obj.data,
        })
    }
}

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

impl <DATA> From<DATA> for NewModel<DATA>
    where
        DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned {
    fn from(data: DATA) -> Self {
        NewModel::new(data)
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

pub trait C3p0Json<DATA, CODEC, DB>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
    CODEC: JsonCodec<DATA>,
    DB: JsonManagerBase<DATA, CODEC>,
{
    fn json_manager(&self) -> &DB;

    fn create_table_if_not_exists(&self, conn: &DB::Conn) -> Result<(), C3p0Error> {
        self.json_manager().create_table_if_not_exists(conn)
    }

    fn drop_table_if_exists(&self, conn: &DB::Conn) -> Result<(), C3p0Error> {
        self.json_manager().drop_table_if_exists(conn)
    }

    fn lock_table_exclusively(&self, conn: &DB::Conn) -> Result<(), C3p0Error> {
        self.json_manager().lock_table_exclusively(conn)
    }

    fn count_all(&self, conn: &DB::Conn) -> Result<IdType, C3p0Error> {
        self.json_manager().count_all(conn)
    }

    fn exists_by_id<'a, ID: Into<&'a IdType>>(
        &'a self,
        conn: &DB::Conn,
        id: ID,
    ) -> Result<bool, C3p0Error> {
        self.json_manager().exists_by_id(conn, *id.into())
    }

    fn find_all(&self, conn: &DB::Conn) -> Result<Vec<Model<DATA>>, C3p0Error> {
        self.json_manager().find_all(conn)
    }

    fn find_by_id<'a, ID: Into<&'a IdType>>(
        &'a self,
        conn: &DB::Conn,
        id: ID,
    ) -> Result<Option<Model<DATA>>, C3p0Error> {
        self.json_manager().find_by_id(conn, *id.into())
    }

    fn delete(&self, conn: &DB::Conn, obj: &Model<DATA>) -> Result<u64, C3p0Error> {
        self.json_manager().delete(conn, obj)
    }

    fn delete_all(&self, conn: &DB::Conn) -> Result<u64, C3p0Error> {
        self.json_manager().delete_all(conn)
    }

    fn delete_by_id<'a, ID: Into<&'a IdType>>(
        &'a self,
        conn: &DB::Conn,
        id: ID,
    ) -> Result<u64, C3p0Error> {
        self.json_manager().delete_by_id(conn, *id.into())
    }

    fn save<M: Into<NewModel<DATA>>>(&self, conn: &DB::Conn, obj: M) -> Result<Model<DATA>, C3p0Error> {
        self.json_manager().save(conn, obj)
    }

    fn update(&self, conn: &DB::Conn, obj: Model<DATA>) -> Result<Model<DATA>, C3p0Error> {
        self.json_manager().update(conn, obj)
    }
}

#[derive(Clone)]
pub struct C3p0JsonRepository<DATA, CODEC, DB>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
    CODEC: JsonCodec<DATA>,
    DB: JsonManagerBase<DATA, CODEC>,
{
    db: DB,
    phantom_data: std::marker::PhantomData<DATA>,
    phantom_codec: std::marker::PhantomData<CODEC>,
}

impl<DATA, CODEC, DB> C3p0JsonRepository<DATA, CODEC, DB>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
    CODEC: JsonCodec<DATA>,
    DB: JsonManagerBase<DATA, CODEC>,
{
    pub fn build(db: DB) -> Self {
        C3p0JsonRepository {
            db,
            phantom_data: std::marker::PhantomData,
            phantom_codec: std::marker::PhantomData,
        }
    }
}

impl<DATA, CODEC, DB> C3p0Json<DATA, CODEC, DB> for C3p0JsonRepository<DATA, CODEC, DB>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
    CODEC: JsonCodec<DATA>,
    DB: JsonManagerBase<DATA, CODEC>,
{
    fn json_manager(&self) -> &DB {
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
