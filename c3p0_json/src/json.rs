use crate::json::codec::JsonCodec;
use crate::json::model::*;
use c3p0_common::error::C3p0Error;
use c3p0_common::pool::Connection;

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

/*
pub trait JsonManagerBase<DATA, CODEC: JsonCodec<DATA>>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
{
    type Conn: ConnectionBase;

    fn codec(&self) -> &CODEC;

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

    fn create_table_if_not_exists(&self, conn: &Self::Conn) -> Result<(), C3p0Error> {
        conn.execute(self.create_table_sql_query(), &[])?;
        Ok(())
    }

    fn drop_table_if_exists(&self, conn: &Self::Conn) -> Result<(), C3p0Error> {
        conn.execute(self.drop_table_sql_query(), &[])?;
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

    fn save(&self, conn: &Self::Conn, obj: NewModel<DATA>) -> Result<Model<DATA>, C3p0Error> {
        let json_data = self.codec().to_value(&obj.data)?;
        let id = conn.fetch_one_value(self.save_sql_query(), &[&obj.version, &json_data])?;
        Ok(Model {
            id,
            version: obj.version,
            data: obj.data,
        })
    }
}
*/

pub trait C3p0Json<DATA, CODEC>
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
