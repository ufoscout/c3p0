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

    fn save(
        &self,
        _conn: &InMemoryConnection,
        obj: NewModel<DATA>,
    ) -> Result<Model<DATA>, C3p0Error> {
        let id = 0;
        Ok(Model {
            id,
            version: obj.version,
            data: obj.data,
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
