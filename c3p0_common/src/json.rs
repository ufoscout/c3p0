use crate::json::codec::JsonCodec;
use crate::json::model::*;
use crate::error::C3p0Error;
use crate::pool::Connection;

pub mod builder;
pub mod codec;
pub mod model;


#[derive(Clone)]
pub struct Queries {
    pub id_field_name: String,
    pub version_field_name: String,
    pub data_field_name: String,

    pub table_name: String,
    pub schema_name: Option<String>,
    pub qualified_table_name: String,

    pub count_all_sql_query: String,
    pub exists_by_id_sql_query: String,

    pub find_all_sql_query: String,
    pub find_by_id_sql_query: String,

    pub delete_sql_query: String,
    pub delete_all_sql_query: String,
    pub delete_by_id_sql_query: String,

    pub save_sql_query: String,

    pub update_sql_query: String,

    pub create_table_sql_query: String,
    pub drop_table_sql_query: String,
    pub lock_table_sql_query: Option<String>,
}



pub trait C3p0JsonManger<DATA, CODEC>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
    CODEC: JsonCodec<DATA>,
{
    type CONNECTION: Connection;

    fn codec(&self) -> &CODEC;

    fn queries(&self) -> &Queries;

    fn create_table_if_not_exists(&self, conn: &Self::CONNECTION) -> Result<(), C3p0Error>;

    fn drop_table_if_exists(&self, conn: &Self::CONNECTION) -> Result<(), C3p0Error>;

    fn count_all(&self, conn: &Self::CONNECTION) -> Result<IdType, C3p0Error>;

    fn exists_by_id<'a, ID: Into<&'a IdType>>(
        &'a self,
        conn: &Self::CONNECTION,
        id: ID,
    ) -> Result<bool, C3p0Error>;

    fn find_all(&self, conn: &Self::CONNECTION) -> Result<Vec<Model<DATA>>, C3p0Error>;

    fn find_by_id<'a, ID: Into<&'a IdType>>(
        &'a self,
        conn: &Self::CONNECTION,
        id: ID,
    ) -> Result<Option<Model<DATA>>, C3p0Error>;

    fn delete(&self, conn: &Self::CONNECTION, obj: &Model<DATA>) -> Result<u64, C3p0Error>;

    fn delete_all(&self, conn: &Self::CONNECTION) -> Result<u64, C3p0Error>;

    fn delete_by_id<'a, ID: Into<&'a IdType>>(
        &'a self,
        conn: &Self::CONNECTION,
        id: ID,
    ) -> Result<u64, C3p0Error>;

    fn save(&self, conn: &Self::CONNECTION, obj: NewModel<DATA>) -> Result<Model<DATA>, C3p0Error>;

    fn update(&self, conn: &Self::CONNECTION, obj: Model<DATA>) -> Result<Model<DATA>, C3p0Error>;
}



pub struct C3p0Json<DATA, CODEC, JSONMANAGER>
    where
        DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
        CODEC: JsonCodec<DATA>,
        JSONMANAGER: C3p0JsonManger<DATA, CODEC>
{
    json_manager: JSONMANAGER,
    phantom_data: std::marker::PhantomData<DATA>,
    phantom_codec: std::marker::PhantomData<CODEC>,
}

impl <DATA, CODEC, JSONMANAGER> C3p0Json<DATA, CODEC, JSONMANAGER>
    where
        DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
        CODEC: JsonCodec<DATA>,
        JSONMANAGER: C3p0JsonManger<DATA, CODEC> {

    pub fn new(json_manager: JSONMANAGER) -> Self {
        C3p0Json{
            json_manager,
            phantom_data: std::marker::PhantomData,
            phantom_codec: std::marker::PhantomData
        }
    }

    pub fn codec(&self) -> &CODEC {
        self.json_manager.codec()
    }

    pub fn queries(&self) -> &Queries {
        self.json_manager.queries()
    }

    pub fn create_table_if_not_exists(&self, conn: &JSONMANAGER::CONNECTION) -> Result<(), C3p0Error> {
        self.json_manager.create_table_if_not_exists(conn)
    }

    pub fn drop_table_if_exists(&self, conn: &JSONMANAGER::CONNECTION) -> Result<(), C3p0Error>{
        self.json_manager.drop_table_if_exists(conn)
    }

    pub fn count_all(&self, conn: &JSONMANAGER::CONNECTION) -> Result<IdType, C3p0Error> {
        self.json_manager.count_all(conn)
    }

    pub fn exists_by_id<'a, ID: Into<&'a IdType>>(
        &'a self,
        conn: &JSONMANAGER::CONNECTION,
        id: ID,
    ) -> Result<bool, C3p0Error> {
        self.json_manager.exists_by_id(conn, id)
    }

    pub fn find_all(&self, conn: &JSONMANAGER::CONNECTION) -> Result<Vec<Model<DATA>>, C3p0Error> {
        self.json_manager.find_all(conn)
    }

    pub fn find_by_id<'a, ID: Into<&'a IdType>>(
        &'a self,
        conn: &JSONMANAGER::CONNECTION,
        id: ID,
    ) -> Result<Option<Model<DATA>>, C3p0Error> {
        self.json_manager.find_by_id(conn, id)
    }

    pub fn delete(&self, conn: &JSONMANAGER::CONNECTION, obj: &Model<DATA>) -> Result<u64, C3p0Error> {
        self.json_manager.delete(conn, obj)
    }

    pub fn delete_all(&self, conn: &JSONMANAGER::CONNECTION) -> Result<u64, C3p0Error> {
        self.json_manager.delete_all(conn)
    }

    pub fn delete_by_id<'a, ID: Into<&'a IdType>>(
        &'a self,
        conn: &JSONMANAGER::CONNECTION,
        id: ID,
    ) -> Result<u64, C3p0Error> {
        self.json_manager.delete_by_id(conn, id)
    }

    pub fn save(&self, conn: &JSONMANAGER::CONNECTION, obj: NewModel<DATA>) -> Result<Model<DATA>, C3p0Error> {
        self.json_manager.save(conn, obj)
    }

    pub fn update(&self, conn: &JSONMANAGER::CONNECTION, obj: Model<DATA>) -> Result<Model<DATA>, C3p0Error> {
        self.json_manager.update(conn, obj)
    }
}
