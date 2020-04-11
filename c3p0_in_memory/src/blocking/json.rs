use crate::blocking::pool::{InMemoryC3p0Pool, InMemoryConnection};
use c3p0_common::blocking::*;
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};

pub trait InMemoryC3p0JsonBuilder {
    fn build<DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send>(
        self,
    ) -> InMemoryC3p0Json<DATA>;
}

impl InMemoryC3p0JsonBuilder for C3p0JsonBuilder<InMemoryC3p0Pool> {
    fn build<DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send>(
        self,
    ) -> InMemoryC3p0Json<DATA> {
        let qualified_table_name = match &self.schema_name {
            Some(schema_name) => format!(r#"{}."{}""#, schema_name, self.table_name),
            None => self.table_name.clone(),
        };
        InMemoryC3p0Json {
            qualified_table_name,
            phantom_data: std::marker::PhantomData,
            codec: Default::default(),
        }
    }
}

#[derive(Clone)]
pub struct InMemoryC3p0Json<DATA>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send,
{
    qualified_table_name: String,
    phantom_data: std::marker::PhantomData<DATA>,
    codec: DefaultJsonCodec,
}

impl<DATA> InMemoryC3p0Json<DATA>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send,
{
    fn get_table<'a>(
        &self,
        qualified_table_name: &str,
        db: &'a HashMap<String, BTreeMap<IdType, Model<Value>>>,
    ) -> Option<&'a BTreeMap<IdType, Model<Value>>> {
        db.get(qualified_table_name)
    }

    fn get_or_create_table<'a>(
        &self,
        qualified_table_name: &str,
        db: &'a mut HashMap<String, BTreeMap<IdType, Model<Value>>>,
    ) -> &'a mut BTreeMap<IdType, Model<Value>> {
        db.entry(qualified_table_name.to_owned())
            .or_insert_with(BTreeMap::new)
    }

    fn to_value_model(&self, model: &Model<DATA>) -> Result<Model<Value>, C3p0Error> {
        Ok(Model {
            id: model.id,
            version: model.version,
            data: serde_json::to_value(&model.data)?,
        })
    }

    fn to_data_model(&self, model: &Model<Value>) -> Result<Model<DATA>, C3p0Error> {
        Ok(Model {
            id: model.id,
            version: model.version,
            data: serde_json::from_value(model.data.clone())?,
        })
    }
}

impl<DATA> C3p0Json<DATA, DefaultJsonCodec> for InMemoryC3p0Json<DATA>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send,
{
    type Conn = InMemoryConnection;

    fn codec(&self) -> &DefaultJsonCodec {
        &self.codec
    }

    fn create_table_if_not_exists(&self, conn: &mut InMemoryConnection) -> Result<(), C3p0Error> {
        conn.write_db(|db| {
            self.get_or_create_table(&self.qualified_table_name, db);
            Ok(())
        })
    }

    fn drop_table_if_exists(
        &self,
        conn: &mut InMemoryConnection,
        _cascade: bool,
    ) -> Result<(), C3p0Error> {
        conn.write_db(|db| {
            db.remove(&self.qualified_table_name);
            Ok(())
        })
    }

    fn count_all(&self, conn: &mut InMemoryConnection) -> Result<u64, C3p0Error> {
        conn.read_db(|db| {
            if let Some(table) = self.get_table(&self.qualified_table_name, db) {
                Ok(table.len() as u64)
            } else {
                Ok(0)
            }
        })
    }

    fn exists_by_id<'a, ID: Into<&'a IdType>>(
        &self,
        conn: &mut InMemoryConnection,
        id: ID,
    ) -> Result<bool, C3p0Error> {
        conn.read_db(|db| {
            if let Some(table) = self.get_table(&self.qualified_table_name, db) {
                Ok(table.contains_key(id.into()))
            } else {
                Ok(false)
            }
        })
    }

    fn fetch_all(&self, conn: &mut InMemoryConnection) -> Result<Vec<Model<DATA>>, C3p0Error> {
        conn.read_db(|db| {
            if let Some(table) = self.get_table(&self.qualified_table_name, db) {
                table
                    .values()
                    .map(|value| self.to_data_model(value))
                    .collect::<Result<Vec<_>, _>>()
            } else {
                Ok(vec![])
            }
        })
    }

    fn fetch_all_for_update(
        &self,
        conn: &mut Self::Conn,
        _for_update: &ForUpdate,
    ) -> Result<Vec<Model<DATA>>, C3p0Error> {
        self.fetch_all(conn)
    }

    fn fetch_one_optional_by_id<'a, ID: Into<&'a IdType>>(
        &self,
        conn: &mut InMemoryConnection,
        id: ID,
    ) -> Result<Option<Model<DATA>>, C3p0Error> {
        conn.read_db(|db| {
            if let Some(table) = self.get_table(&self.qualified_table_name, db) {
                if let Some(value) = table.get(id.into()) {
                    return Ok(Some(self.to_data_model(value)?));
                }
            }
            Ok(None)
        })
    }

    fn fetch_one_optional_by_id_for_update<'a, ID: Into<&'a IdType>>(
        &'a self,
        conn: &mut Self::Conn,
        id: ID,
        _for_update: &ForUpdate,
    ) -> Result<Option<Model<DATA>>, C3p0Error> {
        self.fetch_one_optional_by_id(conn, id)
    }

    fn delete(
        &self,
        conn: &mut InMemoryConnection,
        obj: Model<DATA>,
    ) -> Result<Model<DATA>, C3p0Error> {
        conn.write_db(|db| {
            let table = self.get_or_create_table(&self.qualified_table_name, db);

            let mut good_version = false;

            if let Some(value) = table.get(&obj.id) {
                good_version = value.version == obj.version;
            };

            if good_version {
                table.remove(&obj.id);
                return Ok(obj);
            }

            Err(C3p0Error::OptimisticLockError{ message: format!("Cannot delete data in table [{}] with id [{}], version [{}]: data was changed!",
                                                                 &self.qualified_table_name, &obj.id, &obj.version
            )})

        })
    }

    fn delete_all(&self, conn: &mut InMemoryConnection) -> Result<u64, C3p0Error> {
        conn.write_db(|db| {
            let table = self.get_or_create_table(&self.qualified_table_name, db);
            let len = table.len();
            table.clear();
            Ok(len as u64)
        })
    }

    fn delete_by_id<'a, ID: Into<&'a IdType>>(
        &self,
        conn: &mut InMemoryConnection,
        id: ID,
    ) -> Result<u64, C3p0Error> {
        conn.write_db(|db| {
            let table = self.get_or_create_table(&self.qualified_table_name, db);
            match table.remove(id.into()) {
                Some(_) => Ok(1),
                None => Ok(0),
            }
        })
    }

    fn save(
        &self,
        conn: &mut InMemoryConnection,
        obj: NewModel<DATA>,
    ) -> Result<Model<DATA>, C3p0Error> {
        conn.write_db(|db| {
            let table = self.get_or_create_table(&self.qualified_table_name, db);
            let id = table.len() as IdType;
            let model = Model {
                id,
                version: obj.version,
                data: obj.data,
            };
            table.insert(id, self.to_value_model(&model)?);
            Ok(model)
        })
    }

    fn update(
        &self,
        conn: &mut InMemoryConnection,
        obj: Model<DATA>,
    ) -> Result<Model<DATA>, C3p0Error> {
        conn.write_db(|db| {
            let table = self.get_or_create_table(&self.qualified_table_name, db);

            let mut good_version = false;

            if let Some(value) = table.get(&obj.id) {
                good_version = value.version == obj.version;
            };

            if good_version {
                let updated_model = Model {
                    id: obj.id,
                    version: obj.version + 1,
                    data: obj.data,
                };
                table.insert(updated_model.id, self.to_value_model(&updated_model)?);
                return Ok(updated_model);
            }

            Err(C3p0Error::OptimisticLockError{ message: format!("Cannot update data in table [{}] with id [{}], version [{}]: data was changed!",
                                                                        &self.qualified_table_name, &obj.id, &obj.version
            )})

        })
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use serde_derive::{Deserialize, Serialize};

    #[derive(Clone, Serialize, Deserialize)]
    struct TestData {
        value: String,
    }

    impl TestData {
        fn new(value: &str) -> Self {
            Self {
                value: value.to_string(),
            }
        }
    }

    #[test]
    fn should_save_and_fetch_new_model() -> Result<(), C3p0Error> {
        // Arrange
        let pool = InMemoryC3p0Pool::new();
        let c3p0 = C3p0JsonBuilder::new("TABLE_1").build::<TestData>();

        pool.transaction(|conn| {
            // Act
            let saved_model_1 = c3p0.save(conn, TestData::new("value1").into())?;
            let fetched_model_1 = c3p0.fetch_one_optional_by_id(conn, &saved_model_1)?;
            let exist_model_1 = c3p0.exists_by_id(conn, &saved_model_1)?;

            let saved_model_2 = c3p0.save(conn, TestData::new("value2").into())?;
            let fetched_model_2 = c3p0.fetch_one_optional_by_id(conn, &saved_model_2.id)?;
            let exist_model_2 = c3p0.exists_by_id(conn, &saved_model_2)?;

            // Assert
            assert!(saved_model_2.id > saved_model_1.id);

            assert!(exist_model_1);
            assert_eq!(saved_model_1.data.value, "value1");

            let fetched_model_1 = fetched_model_1.unwrap();
            assert_eq!(saved_model_1.id, fetched_model_1.id);
            assert_eq!(saved_model_1.version, fetched_model_1.version);
            assert_eq!(saved_model_1.data.value, fetched_model_1.data.value);

            assert_eq!(saved_model_2.data.value, "value2");
            assert!(exist_model_2);

            let fetched_model_2 = fetched_model_2.unwrap();
            assert_eq!(saved_model_2.id, fetched_model_2.id);
            assert_eq!(saved_model_2.version, fetched_model_2.version);
            assert_eq!(saved_model_2.data.value, fetched_model_2.data.value);

            Ok(())
        })
    }

    #[test]
    fn should_return_if_exists() -> Result<(), C3p0Error> {
        // Arrange
        let pool = InMemoryC3p0Pool::new();
        let c3p0 = C3p0JsonBuilder::new("TABLE_1").build::<TestData>();

        pool.transaction(|conn| {
            // Act
            let saved_model_1 = c3p0.save(conn, TestData::new("value1").into())?;
            let exist_model_1 = c3p0.exists_by_id(conn, &saved_model_1)?;

            let exist_model_2 = c3p0.exists_by_id(conn, &(saved_model_1.id + 1))?;

            // Assert
            assert!(exist_model_1);
            assert!(!exist_model_2);

            Ok(())
        })
    }

    #[test]
    fn should_count_records() -> Result<(), C3p0Error> {
        // Arrange
        let pool = InMemoryC3p0Pool::new();
        let c3p0_1 = C3p0JsonBuilder::new("TABLE_1").build::<TestData>();
        let c3p0_2a = C3p0JsonBuilder::new("TABLE_2").build::<TestData>();
        let c3p0_2b = C3p0JsonBuilder::new("TABLE_2").build::<TestData>();

        pool.transaction(|conn| {
            // Act
            assert_eq!(0, c3p0_1.count_all(conn)?);
            assert_eq!(0, c3p0_2a.count_all(conn)?);
            assert_eq!(0, c3p0_2b.count_all(conn)?);

            assert!(c3p0_1.save(conn, TestData::new("value1").into()).is_ok());

            assert_eq!(1, c3p0_1.count_all(conn)?);
            assert_eq!(0, c3p0_2a.count_all(conn)?);
            assert_eq!(0, c3p0_2b.count_all(conn)?);

            assert!(c3p0_1.save(conn, TestData::new("value1").into()).is_ok());
            assert!(c3p0_1.save(conn, TestData::new("value1").into()).is_ok());
            assert!(c3p0_2a.save(conn, TestData::new("value1").into()).is_ok());
            assert!(c3p0_2b.save(conn, TestData::new("value1").into()).is_ok());

            assert_eq!(3, c3p0_1.count_all(conn)?);
            assert_eq!(2, c3p0_2a.count_all(conn)?);
            assert_eq!(2, c3p0_2b.count_all(conn)?);

            Ok(())
        })
    }

    #[test]
    fn should_delete_by_id_and_delete_all() -> Result<(), C3p0Error> {
        // Arrange
        let pool = InMemoryC3p0Pool::new();
        let c3p0_1 = C3p0JsonBuilder::new("TABLE_1").build::<TestData>();
        let c3p0_2 = C3p0JsonBuilder::new("TABLE_2").build::<TestData>();

        // Act
        pool.transaction(|conn| {
            assert_eq!(0, c3p0_1.count_all(conn)?);
            assert_eq!(0, c3p0_2.count_all(conn)?);

            assert!(c3p0_1.save(conn, TestData::new("value1").into()).is_ok());
            assert!(c3p0_2.save(conn, TestData::new("value1").into()).is_ok());
            assert!(c3p0_2.save(conn, TestData::new("value1").into()).is_ok());

            let saved_on_2 = c3p0_2.save(conn, TestData::new("value1").into())?;

            assert_eq!(1, c3p0_1.count_all(conn)?);
            assert_eq!(3, c3p0_2.count_all(conn)?);

            assert_eq!(1, c3p0_2.delete_by_id(conn, &saved_on_2.id)?);

            assert!(!c3p0_2.exists_by_id(conn, &saved_on_2.id)?);
            assert_eq!(1, c3p0_1.count_all(conn)?);
            assert_eq!(2, c3p0_2.count_all(conn)?);

            assert_eq!(2, c3p0_2.delete_all(conn)?);

            assert_eq!(1, c3p0_1.count_all(conn)?);
            assert_eq!(0, c3p0_2.count_all(conn)?);

            Ok(())
        })
    }

    #[test]
    fn should_create_and_drop_table() -> Result<(), C3p0Error> {
        // Arrange
        let pool = InMemoryC3p0Pool::new();
        let c3p0_1 = C3p0JsonBuilder::new("TABLE_1").build::<TestData>();

        pool.transaction(|conn| {
            // Act
            assert!(c3p0_1.create_table_if_not_exists(conn).is_ok());
            assert!(c3p0_1.create_table_if_not_exists(conn).is_ok());

            assert!(c3p0_1.save(conn, TestData::new("value1").into()).is_ok());

            assert_eq!(1, c3p0_1.count_all(conn)?);

            assert!(c3p0_1.drop_table_if_exists(conn, false).is_ok());

            assert_eq!(0, c3p0_1.count_all(conn)?);

            Ok(())
        })
    }

    #[test]
    fn should_fetch_all() -> Result<(), C3p0Error> {
        // Arrange
        let pool = InMemoryC3p0Pool::new();
        let c3p0 = C3p0JsonBuilder::new("TABLE_1").build::<TestData>();

        pool.transaction(|conn| {
            // Act
            let saved_model_0 = c3p0.save(conn, TestData::new("value1").into())?;
            let saved_model_1 = c3p0.save(conn, TestData::new("value2").into())?;
            let saved_model_2 = c3p0.save(conn, TestData::new("value2").into())?;

            let all = c3p0.fetch_all(conn)?;

            // Assert
            assert_eq!(3, all.len());

            let fetched_model_0 = all.get(0).unwrap();
            assert_eq!(saved_model_0.id, fetched_model_0.id);
            assert_eq!(saved_model_0.version, fetched_model_0.version);
            assert_eq!(saved_model_0.data.value, fetched_model_0.data.value);

            let fetched_model_1 = all.get(1).unwrap();
            assert_eq!(saved_model_1.id, fetched_model_1.id);
            assert_eq!(saved_model_1.version, fetched_model_1.version);
            assert_eq!(saved_model_1.data.value, fetched_model_1.data.value);

            let fetched_model_2 = all.get(2).unwrap();
            assert_eq!(saved_model_2.id, fetched_model_2.id);
            assert_eq!(saved_model_2.version, fetched_model_2.version);
            assert_eq!(saved_model_2.data.value, fetched_model_2.data.value);

            Ok(())
        })
    }

    #[test]
    fn should_update_with_optimistic_lock() -> Result<(), C3p0Error> {
        // Arrange
        let pool = InMemoryC3p0Pool::new();
        let c3p0 = C3p0JsonBuilder::new("TABLE_1").build::<TestData>();

        pool.transaction(|conn| {
            // Act
            let saved_model = c3p0.save(conn, TestData::new("value1").into())?;
            let updated_model = c3p0.update(conn, saved_model.clone())?;
            let fetched_model = c3p0.fetch_one_optional_by_id(conn, &saved_model)?.unwrap();

            let updated_result_1 = c3p0.update(conn, saved_model.clone());
            let updated_result_2 = c3p0.update(conn, updated_model.clone());

            // Assert
            assert_eq!(saved_model.id, updated_model.id);
            assert_eq!(saved_model.version + 1, updated_model.version);
            assert_eq!(saved_model.data.value, updated_model.data.value);

            assert_eq!(saved_model.id, fetched_model.id);
            assert_eq!(fetched_model.version, updated_model.version);

            assert!(updated_result_2.is_ok());

            match updated_result_1 {
                Err(C3p0Error::OptimisticLockError { .. }) => assert!(true),
                _ => assert!(false),
            }

            Ok(())
        })
    }

    #[test]
    fn should_delete_with_optimistic_lock() -> Result<(), C3p0Error> {
        // Arrange
        let pool = InMemoryC3p0Pool::new();
        let c3p0 = C3p0JsonBuilder::new("TABLE_1").build::<TestData>();

        pool.transaction(|conn| {
            // Act
            let saved_model = c3p0.save(conn, TestData::new("value1").into())?;
            let updated_model = c3p0.update(conn, saved_model.clone())?;
            let fetched_model = c3p0.fetch_one_optional_by_id(conn, &saved_model)?.unwrap();

            let delete_result_1 = c3p0.delete(conn, saved_model.clone());
            assert!(c3p0.exists_by_id(conn, &saved_model)?);

            let delete_result_2 = c3p0.delete(conn, updated_model.clone());
            assert!(!c3p0.exists_by_id(conn, &saved_model)?);

            // Assert
            assert_eq!(saved_model.id, updated_model.id);
            assert_eq!(saved_model.version + 1, updated_model.version);
            assert_eq!(saved_model.data.value, updated_model.data.value);

            assert_eq!(saved_model.id, fetched_model.id);
            assert_eq!(fetched_model.version, updated_model.version);

            assert_eq!(updated_model.id, delete_result_2.unwrap().id);

            match delete_result_1 {
                Err(C3p0Error::OptimisticLockError { .. }) => assert!(true),
                _ => assert!(false),
            }

            Ok(())
        })
    }
}
