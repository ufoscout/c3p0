use crate::json::codec::{JsonCodec, DefaultJsonCodec};
use crate::types::OptString;
use crate::json::{C3p0Json, C3p0JsonManger};
use crate::pool::C3p0PoolManager;

pub trait C3p0JsonBuilderManager<C3P0: C3p0PoolManager> {

    fn build<DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned, CODEC: JsonCodec<DATA>,
    JSONMANAGER: C3p0JsonManger<DATA, CODEC, CONNECTION=C3P0::CONN>>
    (&self, codec: CODEC,
             id_field_name: String,
             version_field_name: String,
             data_field_name: String,
             table_name: String,
             schema_name: Option<String>) -> C3p0Json<DATA, CODEC, JSONMANAGER>;
}

#[derive(Clone)]
pub struct C3p0JsonBuilder<DATA, CODEC: JsonCodec<DATA>, C3P0: C3p0PoolManager, JSONBUILDER: C3p0JsonBuilderManager<C3P0>, JSONMANAGER: C3p0JsonManger<DATA, CODEC, CONNECTION=C3P0::CONN>>
    where
        DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
{
    phantom_data: std::marker::PhantomData<DATA>,
    phantom_c3p0: std::marker::PhantomData<C3P0>,
    phantom_json_manager: std::marker::PhantomData<JSONMANAGER>,
    codec: CODEC,
    json_builder_manager: JSONBUILDER,
    id_field_name: String,
    version_field_name: String,
    data_field_name: String,
    table_name: String,
    schema_name: Option<String>,
}

impl<DATA, C3P0: C3p0PoolManager, JSONBUILDER: C3p0JsonBuilderManager<C3P0>, JSONMANAGER: C3p0JsonManger<DATA, DefaultJsonCodec, CONNECTION=C3P0::CONN>> C3p0JsonBuilder<DATA, DefaultJsonCodec, C3P0, JSONBUILDER, JSONMANAGER>
    where
        DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
{
    pub fn new<T: Into<String>>(json_builder_manager: JSONBUILDER, table_name: T) -> Self {
        let table_name = table_name.into();
        C3p0JsonBuilder {
            phantom_data: std::marker::PhantomData,
            phantom_c3p0: std::marker::PhantomData,
            phantom_json_manager: std::marker::PhantomData,
            codec: DefaultJsonCodec {},
            table_name: table_name.clone(),
            id_field_name: "id".to_owned(),
            version_field_name: "version".to_owned(),
            data_field_name: "data".to_owned(),
            schema_name: None,
            json_builder_manager
        }
    }
}

impl<DATA, CODEC: JsonCodec<DATA>, C3P0: C3p0PoolManager, JSONBUILDER: C3p0JsonBuilderManager<C3P0>, JSONMANAGER: C3p0JsonManger<DATA, CODEC, CONNECTION=C3P0::CONN>> C3p0JsonBuilder<DATA, CODEC, C3P0, JSONBUILDER, JSONMANAGER>
    where
        DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
{
    pub fn with_codec<NEWCODEC: JsonCodec<DATA>, NEWJSONMANAGER: C3p0JsonManger<DATA, NEWCODEC, CONNECTION=C3P0::CONN>>(
        self,
        codec: NEWCODEC,
    ) -> C3p0JsonBuilder<DATA, NEWCODEC, C3P0, JSONBUILDER, NEWJSONMANAGER> {
        C3p0JsonBuilder {
            phantom_data: self.phantom_data,
            phantom_c3p0: self.phantom_c3p0,
            phantom_json_manager: std::marker::PhantomData,
            codec,
            table_name: self.table_name,
            id_field_name: self.id_field_name,
            version_field_name: self.version_field_name,
            data_field_name: self.data_field_name,
            schema_name: self.schema_name,
            json_builder_manager: self.json_builder_manager
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
}