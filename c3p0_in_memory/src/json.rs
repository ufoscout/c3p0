use crate::pool::{InMemoryC3p0Pool, InMemoryConnection};
use c3p0_common::error::C3p0Error;
use c3p0_common::json::builder::C3p0JsonBuilder;
use c3p0_common::json::{
    model::{IdType, Model, NewModel},
    C3p0Json,
};
use c3p0_common::DefaultJsonCodec;
use std::collections::HashMap;
use serde_json::Value;

pub trait InMemoryC3p0JsonBuilder {
    fn build<DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned>(
        self,
    ) -> InMemoryC3p0Json<DATA>;
}

impl InMemoryC3p0JsonBuilder for C3p0JsonBuilder<InMemoryC3p0Pool> {
    fn build<DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned>(
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
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
{
    qualified_table_name: String,
    phantom_data: std::marker::PhantomData<DATA>,
    codec: DefaultJsonCodec,
}

impl<DATA> InMemoryC3p0Json<DATA>
    where
        DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned
{

    fn get_table<'a>(&self, qualified_table_name: &str, db: &'a HashMap<String, HashMap<IdType, Value>>) -> Option<&'a HashMap<IdType, Value>> {
        db.get(qualified_table_name)
    }

    fn get_or_create_table<'a>(&self, qualified_table_name: &str, db: &'a mut HashMap<String, HashMap<IdType, Value>>) -> &'a mut HashMap<IdType, Value> {
        db.entry(qualified_table_name.to_owned()).or_insert_with(|| HashMap::new())
    }

}

impl<DATA> C3p0Json<DATA, DefaultJsonCodec> for InMemoryC3p0Json<DATA>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
{
    type CONN = InMemoryConnection;

    fn codec(&self) -> &DefaultJsonCodec {
        &self.codec
    }

    fn create_table_if_not_exists(&self, conn: &InMemoryConnection) -> Result<(), C3p0Error> {
        Ok(())
    }

    fn drop_table_if_exists(&self, conn: &InMemoryConnection) -> Result<(), C3p0Error> {
        Ok(())
    }

    fn count_all(&self, conn: &InMemoryConnection) -> Result<i64, C3p0Error> {
        conn.read_db(|db| {
            if let Some(table) = self.get_table(&self.qualified_table_name, db) {
               Ok(table.len() as IdType)
            } else {
                Ok(0)
            }
        })
    }

    fn exists_by_id<'a, ID: Into<&'a IdType>>(
        &self,
        conn: &InMemoryConnection,
        id: ID,
    ) -> Result<bool, C3p0Error> {
        conn.read_db(|db| {
            if let Some(table) = self.get_table(&self.qualified_table_name, db) {
                if let Some(value) = table.get(id.into()) {
                    return Ok(true)
                }
            }
            Ok(false)
        })
    }

    fn fetch_all(&self, conn: &InMemoryConnection) -> Result<Vec<Model<DATA>>, C3p0Error> {
        Ok(vec![])
    }

    fn fetch_one_by_id<'a, ID: Into<&'a IdType>>(
        &self,
        conn: &InMemoryConnection,
        id: ID,
    ) -> Result<Option<Model<DATA>>, C3p0Error> {
        conn.read_db(|db| {
            if let Some(table) = self.get_table(&self.qualified_table_name, db) {
                if let Some(value) = table.get(id.into()) {
                    return Ok(serde_json::from_value(value.clone())?)
                }
            }
            Ok(None)
        })
    }

    fn delete(&self, conn: &InMemoryConnection, obj: &Model<DATA>) -> Result<u64, C3p0Error> {
        Ok(0)
    }

    fn delete_all(&self, conn: &InMemoryConnection) -> Result<u64, C3p0Error> {
        conn.write_db(|db| {
            let table = self.get_or_create_table(&self.qualified_table_name, db);
            let len = table.len();
            table.clear();
            Ok(len as u64)
        })
    }

    fn delete_by_id<'a, ID: Into<&'a IdType>>(
        &self,
        conn: &InMemoryConnection,
        id: ID,
    ) -> Result<u64, C3p0Error> {

        conn.write_db(|db| {
            let table = self.get_or_create_table(&self.qualified_table_name, db);
            match table.remove(id.into()) {
                Some(_) => Ok(1),
                None => Ok(0)
            }
        })

    }

    fn save<M: Into<NewModel<DATA>>>(
        &self,
        conn: &InMemoryConnection,
        obj: M,
    ) -> Result<Model<DATA>, C3p0Error> {

        conn.write_db(|db| {
            let table = self.get_or_create_table(&self.qualified_table_name, db);
            let id = table.len() as IdType;
            let new_model = obj.into();
            let model = Model {
                id,
                version: new_model.version,
                data: new_model.data,
            };
            table.insert(id, serde_json::to_value(&model)?);
            Ok(model)
        })

    }

    fn update(
        &self,
        conn: &InMemoryConnection,
        obj: Model<DATA>,
    ) -> Result<Model<DATA>, C3p0Error> {
        Ok(Model {
            id: obj.id,
            version: obj.version + 1,
            data: obj.data,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use c3p0_common::{C3p0Error, C3p0JsonBuilder, C3p0Pool};
    use crate::pool::InMemoryC3p0Pool;
    use serde_derive::{Deserialize, Serialize};

    #[derive(Clone, Serialize, Deserialize)]
    struct TestData {
        value: String
    }

    impl TestData{
        fn new(value: &str) -> Self {
            Self{
                value: value.to_string()
            }
        }
    }

    #[test]
    fn should_save_and_fetch_new_model() -> Result<(), C3p0Error> {
        // Arrange
        let pool = InMemoryC3p0Pool::new();
        let c3p0 = C3p0JsonBuilder::new("TABLE_1").build::<TestData>();

        // Act
        let saved_model_1 = c3p0.save(&pool.connection()?, TestData::new("value1"))?;
        let fetched_model_1 = c3p0.fetch_one_by_id(&pool.connection()?, &saved_model_1)?;
        let exist_model_1 = c3p0.exists_by_id(&pool.connection()?, &saved_model_1)?;

        let saved_model_2 = c3p0.save(&pool.connection()?, TestData::new("value2"))?;
        let fetched_model_2 = c3p0.fetch_one_by_id(&pool.connection()?, &saved_model_2.id)?;
        let exist_model_2 = c3p0.exists_by_id(&pool.connection()?, &saved_model_2)?;

        // Assert
        assert!( saved_model_2.id > saved_model_1.id );

        assert!(exist_model_1);
        assert_eq!( saved_model_1.data.value, "value1" );

        let fetched_model_1 = fetched_model_1.unwrap();
        assert_eq!( saved_model_1.id, fetched_model_1.id );
        assert_eq!( saved_model_1.version, fetched_model_1.version );
        assert_eq!( saved_model_1.data.value, fetched_model_1.data.value );

        assert_eq!( saved_model_2.data.value, "value2" );
        assert!(exist_model_2);

        let fetched_model_2 = fetched_model_2.unwrap();
        assert_eq!( saved_model_2.id, fetched_model_2.id );
        assert_eq!( saved_model_2.version, fetched_model_2.version );
        assert_eq!( saved_model_2.data.value, fetched_model_2.data.value );

        Ok(())
    }

    #[test]
    fn should_return_if_exists() -> Result<(), C3p0Error> {
        // Arrange
        let pool = InMemoryC3p0Pool::new();
        let c3p0 = C3p0JsonBuilder::new("TABLE_1").build::<TestData>();

        // Act
        let saved_model_1 = c3p0.save(&pool.connection()?, TestData::new("value1"))?;
        let exist_model_1 = c3p0.exists_by_id(&pool.connection()?, &saved_model_1)?;

        let exist_model_2 = c3p0.exists_by_id(&pool.connection()?, &(saved_model_1.id + 1))?;

        // Assert
        assert!(exist_model_1);
        assert!(!exist_model_2);

        Ok(())
    }

    #[test]
    fn should_count_records() -> Result<(), C3p0Error> {
        // Arrange
        let pool = InMemoryC3p0Pool::new();
        let c3p0_1 = C3p0JsonBuilder::new("TABLE_1").build::<TestData>();
        let c3p0_2a = C3p0JsonBuilder::new("TABLE_2").build::<TestData>();
        let c3p0_2b = C3p0JsonBuilder::new("TABLE_2").build::<TestData>();

        let conn = pool.connection()?;
        // Act

        assert_eq!(0, c3p0_1.count_all(&conn)?);
        assert_eq!(0, c3p0_2a.count_all(&conn)?);
        assert_eq!(0, c3p0_2b.count_all(&conn)?);

        assert!(c3p0_1.save(&conn, TestData::new("value1")).is_ok());

        assert_eq!(1, c3p0_1.count_all(&conn)?);
        assert_eq!(0, c3p0_2a.count_all(&conn)?);
        assert_eq!(0, c3p0_2b.count_all(&conn)?);

        assert!(c3p0_1.save(&conn, TestData::new("value1")).is_ok());
        assert!(c3p0_1.save(&conn, TestData::new("value1")).is_ok());
        assert!(c3p0_2a.save(&conn, TestData::new("value1")).is_ok());
        assert!(c3p0_2b.save(&conn, TestData::new("value1")).is_ok());

        assert_eq!(3, c3p0_1.count_all(&conn)?);
        assert_eq!(2, c3p0_2a.count_all(&conn)?);
        assert_eq!(2, c3p0_2b.count_all(&conn)?);

        Ok(())
    }

    #[test]
    fn should_delete_by_id_and_delete_all() -> Result<(), C3p0Error> {
        // Arrange
        let pool = InMemoryC3p0Pool::new();
        let c3p0_1 = C3p0JsonBuilder::new("TABLE_1").build::<TestData>();
        let c3p0_2 = C3p0JsonBuilder::new("TABLE_2").build::<TestData>();

        let conn = pool.connection()?;
        // Act

        assert_eq!(0, c3p0_1.count_all(&conn)?);
        assert_eq!(0, c3p0_2.count_all(&conn)?);

        assert!(c3p0_1.save(&conn, TestData::new("value1")).is_ok());
        assert!(c3p0_2.save(&conn, TestData::new("value1")).is_ok());
        assert!(c3p0_2.save(&conn, TestData::new("value1")).is_ok());

        let saved_on_2 = c3p0_2.save(&conn, TestData::new("value1"))?;

        assert_eq!(1, c3p0_1.count_all(&conn)?);
        assert_eq!(3, c3p0_2.count_all(&conn)?);

        assert_eq!(1, c3p0_2.delete_by_id(&conn, &saved_on_2.id)?);

        assert!(!c3p0_2.exists_by_id(&conn, &saved_on_2.id)?);
        assert_eq!(1, c3p0_1.count_all(&conn)?);
        assert_eq!(2, c3p0_2.count_all(&conn)?);

        assert_eq!(2, c3p0_2.delete_all(&conn)?);

        assert_eq!(1, c3p0_1.count_all(&conn)?);
        assert_eq!(0, c3p0_2.count_all(&conn)?);

        Ok(())
    }

}
