use crate::error::C3p0Error;
use crate::json::codec::DefaultJsonCodec;
use crate::json::{codec::JsonCodec, JsonManager, Model, NewModel};
use crate::types::OptString;
use mysql_client::prelude::FromValue;
use mysql_client::{Row};
use crate::client::mysql::pool::MySqlConnection;
use crate::pool::ConnectionBase;

#[derive(Clone)]
pub struct MySqlJsonManager<'a, DATA, CODEC: JsonCodec<DATA>>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
{
    phantom_a: std::marker::PhantomData<&'a ()>,
    phantom_data: std::marker::PhantomData<DATA>,

    pub codec: CODEC,

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
}

#[derive(Clone)]
pub struct MySqlJsonManagerBuilder<DATA, CODEC: JsonCodec<DATA>>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
{
    phantom_data: std::marker::PhantomData<DATA>,
    codec: CODEC,
    id_field_name: String,
    version_field_name: String,
    data_field_name: String,
    table_name: String,
    schema_name: Option<String>,
}

impl<DATA> MySqlJsonManagerBuilder<DATA, DefaultJsonCodec>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
{
    pub fn new<T: Into<String>>(table_name: T) -> Self {
        let table_name = table_name.into();
        MySqlJsonManagerBuilder {
            phantom_data: std::marker::PhantomData,
            codec: DefaultJsonCodec {},
            table_name: table_name.clone(),
            id_field_name: "id".to_owned(),
            version_field_name: "version".to_owned(),
            data_field_name: "data".to_owned(),
            schema_name: None,
        }
    }
}

impl<DATA, CODEC: JsonCodec<DATA>> MySqlJsonManagerBuilder<DATA, CODEC>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
{
    pub fn with_codec<NEWCODEC: JsonCodec<DATA>>(
        self,
        codec: NEWCODEC,
    ) -> MySqlJsonManagerBuilder<DATA, NEWCODEC> {
        MySqlJsonManagerBuilder {
            phantom_data: self.phantom_data,
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
    ) -> MySqlJsonManagerBuilder<DATA, CODEC> {
        self.id_field_name = id_field_name.into();
        self
    }

    pub fn with_version_field_name<T: Into<String>>(
        mut self,
        version_field_name: T,
    ) -> MySqlJsonManagerBuilder<DATA, CODEC> {
        self.version_field_name = version_field_name.into();
        self
    }

    pub fn with_data_field_name<T: Into<String>>(
        mut self,
        data_field_name: T,
    ) -> MySqlJsonManagerBuilder<DATA, CODEC> {
        self.data_field_name = data_field_name.into();
        self
    }

    pub fn with_schema_name<O: Into<OptString>>(
        mut self,
        schema_name: O,
    ) -> MySqlJsonManagerBuilder<DATA, CODEC> {
        self.schema_name = schema_name.into().value;
        self
    }

    pub fn build<'a>(self) -> MySqlJsonManager<'a, DATA, CODEC> {
        let qualified_table_name = match &self.schema_name {
            Some(schema_name) => format!(r#"{}."{}""#, schema_name, self.table_name),
            None => self.table_name.clone(),
        };

        MySqlJsonManager {
            phantom_a: std::marker::PhantomData,
            phantom_data: std::marker::PhantomData,

            count_all_sql_query: format!("SELECT COUNT(*) FROM {}", qualified_table_name,),

            exists_by_id_sql_query: format!(
                "SELECT EXISTS (SELECT 1 FROM {} WHERE {} = ?)",
                qualified_table_name, self.id_field_name,
            ),

            find_all_sql_query: format!(
                "SELECT {}, {}, {} FROM {} ORDER BY {} ASC",
                self.id_field_name,
                self.version_field_name,
                self.data_field_name,
                qualified_table_name,
                self.id_field_name,
            ),

            find_by_id_sql_query: format!(
                "SELECT {}, {}, {} FROM {} WHERE {} = ? LIMIT 1",
                self.id_field_name,
                self.version_field_name,
                self.data_field_name,
                qualified_table_name,
                self.id_field_name,
            ),

            delete_sql_query: format!(
                "DELETE FROM {} WHERE {} = ? AND {} = ?",
                qualified_table_name, self.id_field_name, self.version_field_name,
            ),

            delete_all_sql_query: format!("DELETE FROM {}", qualified_table_name,),

            delete_by_id_sql_query: format!(
                "DELETE FROM {} WHERE {} = ?",
                qualified_table_name, self.id_field_name,
            ),

            save_sql_query: format!(
                "INSERT INTO {} ({}, {}) VALUES (?, ?)",
                qualified_table_name, self.version_field_name, self.data_field_name
            ),

            update_sql_query: format!(
                "UPDATE {} SET {} = ?, {} = ? WHERE {} = ? AND {} = ?",
                qualified_table_name,
                self.version_field_name,
                self.data_field_name,
                self.id_field_name,
                self.version_field_name,
            ),

            create_table_sql_query: format!(
                r#"
                CREATE TABLE IF NOT EXISTS {} (
                    {} BIGINT primary key NOT NULL AUTO_INCREMENT,
                    {} int not null,
                    {} JSON
                )
                "#,
                qualified_table_name,
                self.id_field_name,
                self.version_field_name,
                self.data_field_name
            ),

            drop_table_sql_query: format!("DROP TABLE IF EXISTS {}", qualified_table_name),

            codec: self.codec,
            qualified_table_name,
            table_name: self.table_name,
            id_field_name: self.id_field_name,
            version_field_name: self.version_field_name,
            data_field_name: self.data_field_name,
            schema_name: self.schema_name,
        }
    }
}

impl<'a, DATA, CODEC: JsonCodec<DATA>> MySqlJsonManager<'a, DATA, CODEC>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
{
    fn to_model(&self, row: &Row) -> Result<Model<DATA>, C3p0Error> {
        //id: Some(row.get(self.id_field_name.as_str())),
        //version: row.get(self.version_field_name.as_str()),
        //data: (conf.codec.from_value)(row.get(self.data_field_name.as_str()))?
        let id = get_or_error(&row, 0)?;
        let version = get_or_error(&row, 1)?;
        let data = self.codec.from_value(get_or_error(&row, 2)?)?;
        Ok(Model { id, version, data })
    }
}

impl<'a, DATA, CODEC: JsonCodec<DATA>> JsonManager<DATA> for MySqlJsonManager<'a, DATA, CODEC>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
{
    type Conn = MySqlConnection;
    type Ref = &'a mut Self::Conn;

    fn create_table_if_not_exists(&self, conn: Self::Ref) -> Result<u64, C3p0Error> {
        conn.execute(&self.create_table_sql_query, &[])
    }

    fn drop_table_if_exists(&self, conn: Self::Ref) -> Result<u64, C3p0Error> {
        conn.execute(&self.drop_table_sql_query, &[])
    }

    fn count_all(&self, conn: Self::Ref) -> Result<i64, C3p0Error> {
        conn.fetch_one_value(&self.count_all_sql_query, &[])
    }

    fn exists_by_id(&self, conn: Self::Ref, id: i64) -> Result<bool, C3p0Error> {
        conn.fetch_one_value(&self.exists_by_id_sql_query, &[&id])
    }

    fn find_all(&self, conn: Self::Ref) -> Result<Vec<Model<DATA>>, C3p0Error> {
        conn.fetch_all(&self.find_all_sql_query, &[], |row| Ok(self.to_model(row)?))
    }

    fn find_by_id(&self, conn: Self::Ref, id: i64) -> Result<Option<Model<DATA>>, C3p0Error> {
        conn.fetch_one_option(&self.find_by_id_sql_query, &[&id], |row| Ok(self.to_model(row)?))
    }

    fn delete(&self, conn: Self::Ref, obj: &Model<DATA>) -> Result<u64, C3p0Error> {
        let result = conn.execute(&self.delete_sql_query, &[&obj.id, &obj.version])?;

        if result == 0 {
            return Err(C3p0Error::OptimisticLockError{ message: format!("Cannot update data in table [{}] with id [{}], version [{}]: data was changed!",
                                                                        &self.qualified_table_name, &obj.id, &obj.version
            )});
        }

        Ok(result)
    }

    fn delete_all(&self, conn: Self::Ref) -> Result<u64, C3p0Error> {
        conn.execute(&self.delete_all_sql_query, &[])
    }

    fn delete_by_id(&self, conn: Self::Ref, id: i64) -> Result<u64, C3p0Error> {
        conn.execute(&self.delete_by_id_sql_query, &[&id])
    }

    fn save(&self, conn: Self::Ref, obj: NewModel<DATA>) -> Result<Model<DATA>, C3p0Error> {

        let json_data = self.codec.to_value(&obj.data)?;
        {
            conn.execute(&self.save_sql_query, &[&obj.version, &json_data])?;
        }

        let id = {
            conn.fetch_one_value("SELECT LAST_INSERT_ID()", &[])?
        };

        Ok(Model {
            id,
            version: obj.version,
            data: obj.data,
        })
    }

    fn update(&self, conn: Self::Ref, obj: Model<DATA>) -> Result<Model<DATA>, C3p0Error> {
        let json_data = self.codec.to_value(&obj.data)?;

        let updated_model = Model {
            id: obj.id,
            version: obj.version + 1,
            data: obj.data,
        };

        let result = conn.execute(&self.update_sql_query, &[
            &updated_model.version,
            &json_data,
            &updated_model.id,
            &obj.version,
        ])?;

        if result == 0 {
            return Err(C3p0Error::OptimisticLockError{ message: format!("Cannot update data in table [{}] with id [{}], version [{}]: data was changed!",
                                                                        &self.qualified_table_name, &updated_model.id, &obj.version
            )});
        }

        Ok(updated_model)
    }
}

fn get_or_error<T: FromValue>(row: &Row, index: usize) -> Result<T, C3p0Error> {
    row.get(index).ok_or_else(|| C3p0Error::SqlError {
        cause: format!("Row contains no values for index {}", index),
    })
}
