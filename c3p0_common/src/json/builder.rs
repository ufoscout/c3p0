use crate::json::codec::{JsonCodec, DefaultJsonCodec};
use crate::types::OptString;
use crate::json::{C3p0Json, C3p0JsonManager};
use crate::pool::C3p0PoolManager;

#[derive(Clone)]
pub struct C3p0JsonBuilder<DATA, CODEC: JsonCodec<DATA>, POOLMANAGER: C3p0PoolManager>
    where
        DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
{
    phantom_data: std::marker::PhantomData<DATA>,
    phantom_c3p0_pool_manager: std::marker::PhantomData<POOLMANAGER>,
    pub codec: CODEC,
    pub id_field_name: String,
    pub version_field_name: String,
    pub data_field_name: String,
    pub table_name: String,
    pub schema_name: Option<String>,
}

impl<DATA, POOLMANAGER: C3p0PoolManager> C3p0JsonBuilder<DATA, DefaultJsonCodec, POOLMANAGER>
    where
        DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
{
    pub fn new<T: Into<String>>(table_name: T) -> Self {
        let table_name = table_name.into();
        C3p0JsonBuilder {
            phantom_data: std::marker::PhantomData,
            phantom_c3p0_pool_manager: std::marker::PhantomData,
            codec: DefaultJsonCodec {},
            table_name: table_name.clone(),
            id_field_name: "id".to_owned(),
            version_field_name: "version".to_owned(),
            data_field_name: "data".to_owned(),
            schema_name: None
        }
    }
}

impl<DATA, CODEC: JsonCodec<DATA>, POOLMANAGER: C3p0PoolManager> C3p0JsonBuilder<DATA, CODEC, POOLMANAGER>
    where
        DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
{
    pub fn with_codec<NEWCODEC: JsonCodec<DATA>>(
        self,
        codec: NEWCODEC,
    ) -> C3p0JsonBuilder<DATA, NEWCODEC, POOLMANAGER> {
        C3p0JsonBuilder {
            phantom_data: self.phantom_data,
            phantom_c3p0_pool_manager: std::marker::PhantomData,
            codec,
            table_name: self.table_name,
            id_field_name: self.id_field_name,
            version_field_name: self.version_field_name,
            data_field_name: self.data_field_name,
            schema_name: self.schema_name,
        }
    }

    pub fn with_id_field_name<T: Into<String>>(
        mut self,
        id_field_name: T,
    ) -> Self {
        self.id_field_name = id_field_name.into();
        self
    }

    pub fn with_version_field_name<T: Into<String>>(
        mut self,
        version_field_name: T,
    ) -> Self {
        self.version_field_name = version_field_name.into();
        self
    }

    pub fn with_data_field_name<T: Into<String>>(
        mut self,
        data_field_name: T,
    ) -> Self {
        self.data_field_name = data_field_name.into();
        self
    }

    pub fn with_schema_name<O: Into<OptString>>(
        mut self,
        schema_name: O,
    ) -> Self {
        self.schema_name = schema_name.into().value;
        self
    }

    /*
    pub fn build(self) -> C3p0Json<DATA, CODEC, JSONMANAGER> {
        self.json_builder_manager.build(
            self.codec,
            self.id_field_name,
            self.version_field_name,
            self.data_field_name,
            self.table_name,
            self.schema_name
        )
    }
    */
}