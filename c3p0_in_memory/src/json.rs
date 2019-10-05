use crate::pool::{InMemoryC3p0Pool, InMemoryConnection};
use c3p0_common::error::C3p0Error;
use c3p0_common::json::builder::C3p0JsonBuilder;
use c3p0_common::json::{
    model::{IdType, Model, NewModel},
    C3p0Json,
};
use c3p0_common::DefaultJsonCodec;

pub trait InMemoryC3p0JsonBuilder {
    fn build<DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned>(
        self,
    ) -> InMemoryC3p0Json<DATA>;
}

impl InMemoryC3p0JsonBuilder for C3p0JsonBuilder<InMemoryC3p0Pool> {
    fn build<DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned>(
        self,
    ) -> InMemoryC3p0Json<DATA> {
        InMemoryC3p0Json {
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
    phantom_data: std::marker::PhantomData<DATA>,
    codec: DefaultJsonCodec,
}

impl<DATA> C3p0Json<DATA, DefaultJsonCodec> for InMemoryC3p0Json<DATA>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
{
    type CONN = InMemoryConnection;

    fn codec(&self) -> &DefaultJsonCodec {
        &self.codec
    }

    fn create_table_if_not_exists(&self, _conn: &InMemoryConnection) -> Result<(), C3p0Error> {
        Ok(())
    }

    fn drop_table_if_exists(&self, _conn: &InMemoryConnection) -> Result<(), C3p0Error> {
        Ok(())
    }

    fn count_all(&self, _conn: &InMemoryConnection) -> Result<i64, C3p0Error> {
        Ok(0)
    }

    fn exists_by_id<'a, ID: Into<&'a IdType>>(
        &self,
        conn: &InMemoryConnection,
        id: ID,
    ) -> Result<bool, C3p0Error> {
        Ok(false)
    }

    fn fetch_all(&self, _conn: &InMemoryConnection) -> Result<Vec<Model<DATA>>, C3p0Error> {
        Ok(vec![])
    }

    fn fetch_one_by_id<'a, ID: Into<&'a IdType>>(
        &self,
        _conn: &InMemoryConnection,
        id: ID,
    ) -> Result<Option<Model<DATA>>, C3p0Error> {
        Ok(None)
    }

    fn delete(&self, _conn: &InMemoryConnection, obj: &Model<DATA>) -> Result<u64, C3p0Error> {
        Ok(0)
    }

    fn delete_all(&self, _conn: &InMemoryConnection) -> Result<u64, C3p0Error> {
        Ok(0)
    }

    fn delete_by_id<'a, ID: Into<&'a IdType>>(
        &self,
        _conn: &InMemoryConnection,
        id: ID,
    ) -> Result<u64, C3p0Error> {
        Ok(0)
    }

    fn save<M: Into<NewModel<DATA>>>(
        &self,
        _conn: &InMemoryConnection,
        obj: M,
    ) -> Result<Model<DATA>, C3p0Error> {
        let id = 0;
        let new_model = obj.into();
        Ok(Model {
            id,
            version: new_model.version,
            data: new_model.data,
        })
    }

    fn update(
        &self,
        _conn: &InMemoryConnection,
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
    fn should_save_new_model() -> Result<(), C3p0Error> {
        // Arrange
        let pool = InMemoryC3p0Pool::new();
        let c3p0 = C3p0JsonBuilder::new("TABLE_1").build::<TestData>();

        // Act
        let saved_model_1 = c3p0.save(&pool.connection()?, TestData::new("value1"))?;
        let fetched_model_1 = c3p0.fetch_one_by_id(&pool.connection()?, &saved_model_1)?;

        let saved_model_2 = c3p0.save(&pool.connection()?, TestData::new("value2"))?;
        let fetched_model_2 = c3p0.fetch_one_by_id(&pool.connection()?, &saved_model_2.id)?;

        // Assert
        assert!( saved_model_2.id > saved_model_1.id );

        assert_eq!( saved_model_1.data.value, "value1" );
        let fetched_model_1 = fetched_model_1.unwrap();
        assert_eq!( saved_model_1.id, fetched_model_1.id );
        assert_eq!( saved_model_1.version, fetched_model_1.version );
        assert_eq!( saved_model_1.data.value, fetched_model_1.data.value );

        assert_eq!( saved_model_2.data.value, "value2" );
        let fetched_model_2 = fetched_model_2.unwrap();
        assert_eq!( saved_model_2.id, fetched_model_2.id );
        assert_eq!( saved_model_2.version, fetched_model_2.version );
        assert_eq!( saved_model_2.data.value, fetched_model_2.data.value );

        Ok(())
    }


}
