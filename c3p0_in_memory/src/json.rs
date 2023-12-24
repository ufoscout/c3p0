use crate::pool::{InMemoryC3p0Pool, InMemoryConnection};
use async_trait::async_trait;
use c3p0_common::{
    time::utils::get_current_epoch_millis, C3p0Error, C3p0Json, C3p0JsonBuilder, DefaultJsonCodec,
    IdType, Model, NewModel,
};
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};

pub trait InMemoryC3p0JsonBuilder {
    fn build<DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send + Sync>(
        self,
    ) -> InMemoryC3p0Json<DATA>;
}

impl InMemoryC3p0JsonBuilder for C3p0JsonBuilder<InMemoryC3p0Pool> {
    fn build<DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send + Sync>(
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
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send + Sync,
{
    qualified_table_name: String,
    phantom_data: std::marker::PhantomData<DATA>,
    codec: DefaultJsonCodec,
}

impl<DATA> InMemoryC3p0Json<DATA>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send + Sync,
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
        db.entry(qualified_table_name.to_owned()).or_default()
    }

    fn to_value_model(&self, model: &Model<DATA>) -> Result<Model<Value>, C3p0Error> {
        Ok(Model {
            id: model.id,
            version: model.version,
            create_epoch_millis: model.create_epoch_millis,
            update_epoch_millis: model.update_epoch_millis,
            data: serde_json::to_value(&model.data)?,
        })
    }

    fn to_data_model(&self, model: &Model<Value>) -> Result<Model<DATA>, C3p0Error> {
        Ok(Model {
            id: model.id,
            version: model.version,
            create_epoch_millis: model.create_epoch_millis,
            update_epoch_millis: model.update_epoch_millis,
            data: serde_json::from_value(model.data.clone())?,
        })
    }
}

#[async_trait]
impl<DATA> C3p0Json<DATA, DefaultJsonCodec> for InMemoryC3p0Json<DATA>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send + Sync,
{
    type Tx = InMemoryConnection;

    fn codec(&self) -> &DefaultJsonCodec {
        &self.codec
    }

    async fn create_table_if_not_exists(
        &self,
        conn: &mut InMemoryConnection,
    ) -> Result<(), C3p0Error> {
        self.get_or_create_table(&self.qualified_table_name, conn);
        Ok(())
    }

    async fn drop_table_if_exists(
        &self,
        conn: &mut InMemoryConnection,
        _cascade: bool,
    ) -> Result<(), C3p0Error> {
        conn.remove(&self.qualified_table_name);
        Ok(())
    }

    async fn count_all(&self, conn: &mut InMemoryConnection) -> Result<u64, C3p0Error> {
        if let Some(table) = self.get_table(&self.qualified_table_name, conn) {
            Ok(table.len() as u64)
        } else {
            Ok(0)
        }
    }

    async fn exists_by_id<'a, ID: Into<&'a IdType> + Send>(
        &'a self,
        conn: &mut InMemoryConnection,
        id: ID,
    ) -> Result<bool, C3p0Error> {
        if let Some(table) = self.get_table(&self.qualified_table_name, conn) {
            Ok(table.contains_key(id.into()))
        } else {
            Ok(false)
        }
    }

    async fn fetch_all(
        &self,
        conn: &mut InMemoryConnection,
    ) -> Result<Vec<Model<DATA>>, C3p0Error> {
        if let Some(table) = self.get_table(&self.qualified_table_name, conn) {
            table
                .values()
                .map(|value| self.to_data_model(value))
                .collect::<Result<Vec<_>, _>>()
        } else {
            Ok(vec![])
        }
    }

    async fn fetch_one_optional_by_id<'a, ID: Into<&'a IdType> + Send>(
        &'a self,
        conn: &mut InMemoryConnection,
        id: ID,
    ) -> Result<Option<Model<DATA>>, C3p0Error> {
        if let Some(table) = self.get_table(&self.qualified_table_name, conn) {
            if let Some(value) = table.get(id.into()) {
                return Ok(Some(self.to_data_model(value)?));
            }
        }
        Ok(None)
    }

    async fn fetch_one_by_id<'a, ID: Into<&'a IdType> + Send>(
        &'a self,
        conn: &mut Self::Tx,
        id: ID,
    ) -> Result<Model<DATA>, C3p0Error> {
        self.fetch_one_optional_by_id(conn, id)
            .await
            .and_then(|result| result.ok_or(C3p0Error::ResultNotFoundError))
    }

    async fn delete(
        &self,
        conn: &mut InMemoryConnection,
        obj: Model<DATA>,
    ) -> Result<Model<DATA>, C3p0Error> {
        let table = self.get_or_create_table(&self.qualified_table_name, conn);

        let mut good_version = false;

        if let Some(value) = table.get(&obj.id) {
            good_version = value.version == obj.version;
        };

        if good_version {
            table.remove(&obj.id);
            return Ok(obj);
        }

        Err(C3p0Error::OptimisticLockError {
            cause: format!(
                "Cannot delete data in table [{}] with id [{}], version [{}]: data was changed!",
                &self.qualified_table_name, &obj.id, &obj.version
            ),
        })
    }

    async fn delete_all(&self, conn: &mut InMemoryConnection) -> Result<u64, C3p0Error> {
        let table = self.get_or_create_table(&self.qualified_table_name, conn);
        let len = table.len();
        table.clear();
        Ok(len as u64)
    }

    async fn delete_by_id<'a, ID: Into<&'a IdType> + Send>(
        &'a self,
        conn: &mut InMemoryConnection,
        id: ID,
    ) -> Result<u64, C3p0Error> {
        let table = self.get_or_create_table(&self.qualified_table_name, conn);
        match table.remove(id.into()) {
            Some(_) => Ok(1),
            None => Ok(0),
        }
    }

    async fn save(
        &self,
        conn: &mut InMemoryConnection,
        obj: NewModel<DATA>,
    ) -> Result<Model<DATA>, C3p0Error> {
        let table = self.get_or_create_table(&self.qualified_table_name, conn);
        let id = table.len() as IdType;
        let model = Model::from_new(id, get_current_epoch_millis(), obj);
        table.insert(id, self.to_value_model(&model)?);
        Ok(model)
    }

    async fn update(
        &self,
        conn: &mut InMemoryConnection,
        obj: Model<DATA>,
    ) -> Result<Model<DATA>, C3p0Error> {
        let table = self.get_or_create_table(&self.qualified_table_name, conn);

        let mut good_version = false;

        if let Some(value) = table.get(&obj.id) {
            good_version = value.version == obj.version;
        };

        if good_version {
            let updated_model = obj.into_new_version(get_current_epoch_millis());
            table.insert(updated_model.id, self.to_value_model(&updated_model)?);
            return Ok(updated_model);
        }

        Err(C3p0Error::OptimisticLockError {
            cause: format!(
                "Cannot update data in table [{}] with id [{}], version [{}]: data was changed!",
                &self.qualified_table_name, &obj.id, &obj.version
            ),
        })
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use c3p0_common::*;
    use serde::{Deserialize, Serialize};

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

    #[tokio::test]
    async fn should_save_and_fetch_new_model() -> Result<(), C3p0Error> {
        // Arrange
        let pool = InMemoryC3p0Pool::new();
        let c3p0 = C3p0JsonBuilder::new("TABLE_1").build::<TestData>();

        pool.transaction(|conn| async {
            // Act
            let saved_model_1 = c3p0.save(conn, TestData::new("value1").into()).await?;
            let fetched_model_1 = c3p0.fetch_one_optional_by_id(conn, &saved_model_1).await?;
            let exist_model_1 = c3p0.exists_by_id(conn, &saved_model_1).await?;

            let saved_model_2 = c3p0.save(conn, TestData::new("value2").into()).await?;
            let fetched_model_2 = c3p0
                .fetch_one_optional_by_id(conn, &saved_model_2.id)
                .await?;
            let exist_model_2 = c3p0.exists_by_id(conn, &saved_model_2).await?;

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
        .await
    }

    #[tokio::test]
    async fn should_return_if_exists() -> Result<(), C3p0Error> {
        // Arrange
        let pool = InMemoryC3p0Pool::new();
        let c3p0 = C3p0JsonBuilder::new("TABLE_1").build::<TestData>();

        pool.transaction(|conn| async {
            // Act
            let saved_model_1 = c3p0.save(conn, TestData::new("value1").into()).await?;
            let exist_model_1 = c3p0.exists_by_id(conn, &saved_model_1).await?;

            let exist_model_2 = c3p0.exists_by_id(conn, &(saved_model_1.id + 1)).await?;

            // Assert
            assert!(exist_model_1);
            assert!(!exist_model_2);

            Ok(())
        })
        .await
    }

    #[tokio::test]
    async fn should_count_records() -> Result<(), C3p0Error> {
        // Arrange
        let pool = InMemoryC3p0Pool::new();
        let c3p0_1 = C3p0JsonBuilder::new("TABLE_1").build::<TestData>();
        let c3p0_2a = C3p0JsonBuilder::new("TABLE_2").build::<TestData>();
        let c3p0_2b = C3p0JsonBuilder::new("TABLE_2").build::<TestData>();

        pool.transaction(|conn| async {
            // Act
            assert_eq!(0, c3p0_1.count_all(conn).await?);
            assert_eq!(0, c3p0_2a.count_all(conn).await?);
            assert_eq!(0, c3p0_2b.count_all(conn).await?);

            assert!(c3p0_1
                .save(conn, TestData::new("value1").into())
                .await
                .is_ok());

            assert_eq!(1, c3p0_1.count_all(conn).await?);
            assert_eq!(0, c3p0_2a.count_all(conn).await?);
            assert_eq!(0, c3p0_2b.count_all(conn).await?);

            assert!(c3p0_1
                .save(conn, TestData::new("value1").into())
                .await
                .is_ok());
            assert!(c3p0_1
                .save(conn, TestData::new("value1").into())
                .await
                .is_ok());
            assert!(c3p0_2a
                .save(conn, TestData::new("value1").into())
                .await
                .is_ok());
            assert!(c3p0_2b
                .save(conn, TestData::new("value1").into())
                .await
                .is_ok());

            assert_eq!(3, c3p0_1.count_all(conn).await?);
            assert_eq!(2, c3p0_2a.count_all(conn).await?);
            assert_eq!(2, c3p0_2b.count_all(conn).await?);

            Ok(())
        })
        .await
    }

    #[tokio::test]
    async fn should_delete_by_id_and_delete_all() -> Result<(), C3p0Error> {
        // Arrange
        let pool = InMemoryC3p0Pool::new();
        let c3p0_1 = C3p0JsonBuilder::new("TABLE_1").build::<TestData>();
        let c3p0_2 = C3p0JsonBuilder::new("TABLE_2").build::<TestData>();

        // Act
        pool.transaction(|conn| async {
            assert_eq!(0, c3p0_1.count_all(conn).await?);
            assert_eq!(0, c3p0_2.count_all(conn).await?);

            assert!(c3p0_1
                .save(conn, TestData::new("value1").into())
                .await
                .is_ok());
            assert!(c3p0_2
                .save(conn, TestData::new("value1").into())
                .await
                .is_ok());
            assert!(c3p0_2
                .save(conn, TestData::new("value1").into())
                .await
                .is_ok());

            let saved_on_2 = c3p0_2.save(conn, TestData::new("value1").into()).await?;

            assert_eq!(1, c3p0_1.count_all(conn).await?);
            assert_eq!(3, c3p0_2.count_all(conn).await?);

            assert_eq!(1, c3p0_2.delete_by_id(conn, &saved_on_2.id).await?);

            assert!(!c3p0_2.exists_by_id(conn, &saved_on_2.id).await?);
            assert_eq!(1, c3p0_1.count_all(conn).await?);
            assert_eq!(2, c3p0_2.count_all(conn).await?);

            assert_eq!(2, c3p0_2.delete_all(conn).await?);

            assert_eq!(1, c3p0_1.count_all(conn).await?);
            assert_eq!(0, c3p0_2.count_all(conn).await?);

            Ok(())
        })
        .await
    }

    #[tokio::test]
    async fn should_create_and_drop_table() -> Result<(), C3p0Error> {
        // Arrange
        let pool = InMemoryC3p0Pool::new();
        let c3p0_1 = C3p0JsonBuilder::new("TABLE_1").build::<TestData>();

        pool.transaction(|conn| async {
            // Act
            assert!(c3p0_1.create_table_if_not_exists(conn).await.is_ok());
            assert!(c3p0_1.create_table_if_not_exists(conn).await.is_ok());

            assert!(c3p0_1
                .save(conn, TestData::new("value1").into())
                .await
                .is_ok());

            assert_eq!(1, c3p0_1.count_all(conn).await?);

            assert!(c3p0_1.drop_table_if_exists(conn, false).await.is_ok());

            assert_eq!(0, c3p0_1.count_all(conn).await?);

            Ok(())
        })
        .await
    }

    #[tokio::test]
    async fn should_fetch_all() -> Result<(), C3p0Error> {
        // Arrange
        let pool = InMemoryC3p0Pool::new();
        let c3p0 = C3p0JsonBuilder::new("TABLE_1").build::<TestData>();

        pool.transaction(|conn| async {
            // Act
            let saved_model_0 = c3p0.save(conn, TestData::new("value1").into()).await?;
            let saved_model_1 = c3p0.save(conn, TestData::new("value2").into()).await?;
            let saved_model_2 = c3p0.save(conn, TestData::new("value2").into()).await?;

            let all = c3p0.fetch_all(conn).await?;

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
        .await
    }

    #[tokio::test]
    async fn should_update_with_optimistic_lock() -> Result<(), C3p0Error> {
        // Arrange
        let pool = InMemoryC3p0Pool::new();
        let c3p0 = C3p0JsonBuilder::new("TABLE_1").build::<TestData>();

        pool.transaction(|conn| async {
            // Act
            let saved_model = c3p0.save(conn, TestData::new("value1").into()).await?;
            let updated_model = c3p0.update(conn, saved_model.clone()).await?;
            let fetched_model = c3p0
                .fetch_one_optional_by_id(conn, &saved_model)
                .await?
                .unwrap();

            let updated_result_1 = c3p0.update(conn, saved_model.clone()).await;
            let updated_result_2 = c3p0.update(conn, updated_model.clone()).await;

            // Assert
            assert_eq!(saved_model.id, updated_model.id);
            assert_eq!(saved_model.version + 1, updated_model.version);
            assert_eq!(saved_model.data.value, updated_model.data.value);

            assert_eq!(saved_model.id, fetched_model.id);
            assert_eq!(fetched_model.version, updated_model.version);

            assert!(updated_result_2.is_ok());

            match updated_result_1 {
                Err(C3p0Error::OptimisticLockError { .. }) => (),
                _ => panic!(),
            }

            Ok(())
        })
        .await
    }

    #[tokio::test]
    async fn should_delete_with_optimistic_lock() -> Result<(), C3p0Error> {
        // Arrange
        let pool = InMemoryC3p0Pool::new();
        let c3p0 = C3p0JsonBuilder::new("TABLE_1").build::<TestData>();

        pool.transaction(|conn| async {
            // Act
            let saved_model = c3p0.save(conn, TestData::new("value1").into()).await?;
            let updated_model = c3p0.update(conn, saved_model.clone()).await?;
            let fetched_model = c3p0
                .fetch_one_optional_by_id(conn, &saved_model)
                .await?
                .unwrap();

            let delete_result_1 = c3p0.delete(conn, saved_model.clone()).await;
            assert!(c3p0.exists_by_id(conn, &saved_model).await?);

            let delete_result_2 = c3p0.delete(conn, updated_model.clone()).await;
            assert!(!c3p0.exists_by_id(conn, &saved_model).await?);

            // Assert
            assert_eq!(saved_model.id, updated_model.id);
            assert_eq!(saved_model.version + 1, updated_model.version);
            assert_eq!(saved_model.data.value, updated_model.data.value);

            assert_eq!(saved_model.id, fetched_model.id);
            assert_eq!(fetched_model.version, updated_model.version);

            assert_eq!(updated_model.id, delete_result_2.unwrap().id);

            match delete_result_1 {
                Err(C3p0Error::OptimisticLockError { .. }) => (),
                _ => panic!(),
            }

            Ok(())
        })
        .await
    }
}
