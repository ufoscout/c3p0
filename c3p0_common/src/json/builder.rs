use crate::pool::C3p0Pool;
use crate::types::OptString;

#[derive(Clone)]
pub struct C3p0JsonBuilder<C3P0: C3p0Pool> {
    phantom_c3p0_manager: std::marker::PhantomData<C3P0>,
    pub id_field_name: String,
    pub version_field_name: String,
    pub data_field_name: String,
    pub table_name: String,
    pub schema_name: Option<String>,
}

impl<C3P0: C3p0Pool> C3p0JsonBuilder<C3P0> {
    pub fn new<T: Into<String>>(table_name: T) -> Self {
        let table_name = table_name.into();
        C3p0JsonBuilder {
            phantom_c3p0_manager: std::marker::PhantomData,
            table_name,
            id_field_name: "id".to_owned(),
            version_field_name: "version".to_owned(),
            data_field_name: "data".to_owned(),
            schema_name: None,
        }
    }

    pub fn with_id_field_name<T: Into<String>>(mut self, id_field_name: T) -> Self {
        self.id_field_name = id_field_name.into();
        self
    }

    pub fn with_version_field_name<T: Into<String>>(mut self, version_field_name: T) -> Self {
        self.version_field_name = version_field_name.into();
        self
    }

    pub fn with_data_field_name<T: Into<String>>(mut self, data_field_name: T) -> Self {
        self.data_field_name = data_field_name.into();
        self
    }

    pub fn with_schema_name<O: Into<OptString>>(mut self, schema_name: O) -> Self {
        self.schema_name = schema_name.into().value;
        self
    }
}
