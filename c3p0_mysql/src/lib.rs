use crate::error::into_c3p0_error;
use c3p0::codec::Codec;
use c3p0::error::C3p0Error;
use c3p0::manager::DbManager;
use c3p0::types::OptString;
use c3p0::{Model, NewModel};
use mysql::prelude::FromValue;
use mysql::{params, Row};

pub mod error;
pub mod pool;

#[derive(Clone)]
pub struct MySqlManager<'a, DATA>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
{
    phantom_data: std::marker::PhantomData<&'a ()>,
    pub codec: Codec<DATA>,

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
pub struct MySqlManagerBuilder<DATA>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
{
    codec: Codec<DATA>,
    id_field_name: String,
    version_field_name: String,
    data_field_name: String,
    table_name: String,
    schema_name: Option<String>,
}

impl<DATA> MySqlManagerBuilder<DATA>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
{
    pub fn new<T: Into<String>>(table_name: T) -> Self {
        let table_name = table_name.into();
        MySqlManagerBuilder {
            codec: Default::default(),
            table_name: table_name.clone(),
            id_field_name: "id".to_owned(),
            version_field_name: "version".to_owned(),
            data_field_name: "data".to_owned(),
            schema_name: None,
        }
    }

    pub fn with_codec(mut self, codec: Codec<DATA>) -> MySqlManagerBuilder<DATA> {
        self.codec = codec;
        self
    }

    pub fn with_id_field_name<T: Into<String>>(
        mut self,
        id_field_name: T,
    ) -> MySqlManagerBuilder<DATA> {
        self.id_field_name = id_field_name.into();
        self
    }

    pub fn with_version_field_name<T: Into<String>>(
        mut self,
        version_field_name: T,
    ) -> MySqlManagerBuilder<DATA> {
        self.version_field_name = version_field_name.into();
        self
    }

    pub fn with_data_field_name<T: Into<String>>(
        mut self,
        data_field_name: T,
    ) -> MySqlManagerBuilder<DATA> {
        self.data_field_name = data_field_name.into();
        self
    }

    pub fn with_schema_name<O: Into<OptString>>(
        mut self,
        schema_name: O,
    ) -> MySqlManagerBuilder<DATA> {
        self.schema_name = schema_name.into().value;
        self
    }

    pub fn build<'a>(self) -> MySqlManager<'a, DATA> {
        let qualified_table_name = match &self.schema_name {
            Some(schema_name) => format!(r#"{}."{}""#, schema_name, self.table_name),
            None => self.table_name.clone(),
        };

        MySqlManager {
            phantom_data: std::marker::PhantomData,
            count_all_sql_query: format!("SELECT COUNT(*) FROM {}", qualified_table_name,),

            exists_by_id_sql_query: format!(
                "SELECT EXISTS (SELECT 1 FROM {} WHERE {} = :id)",
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
                "SELECT {}, {}, {} FROM {} WHERE {} = :id LIMIT 1",
                self.id_field_name,
                self.version_field_name,
                self.data_field_name,
                qualified_table_name,
                self.id_field_name,
            ),

            delete_sql_query: format!(
                "DELETE FROM {} WHERE {} = :id AND {} = :version",
                qualified_table_name, self.id_field_name, self.version_field_name,
            ),

            delete_all_sql_query: format!("DELETE FROM {}", qualified_table_name,),

            delete_by_id_sql_query: format!(
                "DELETE FROM {} WHERE {} = :id",
                qualified_table_name, self.id_field_name,
            ),

            save_sql_query: format!(
                "INSERT INTO {} ({}, {}) VALUES (:version, :data)",
                qualified_table_name, self.version_field_name, self.data_field_name
            ),

            update_sql_query: format!(
                "UPDATE {} SET {} = :updated_version, {} = :data WHERE {} = :id AND {} = :version",
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

impl<'a, DATA> MySqlManager<'a, DATA>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
{
    fn to_model(&self, row: Row) -> Result<Model<DATA>, C3p0Error> {
        //id: Some(row.get(self.id_field_name.as_str())),
        //version: row.get(self.version_field_name.as_str()),
        //data: (conf.codec.from_value)(row.get(self.data_field_name.as_str()))?
        let id = get_or_error(&row, 0)?;
        let version = get_or_error(&row, 1)?;
        let data = (self.codec.from_value)(get_or_error(&row, 2)?)?;
        Ok(Model { id, version, data })
    }
}

impl<'a, DATA> DbManager<DATA> for MySqlManager<'a, DATA>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
{
    type Conn = mysql::Conn;
    type Ref = &'a mut Self::Conn;

    fn create_table_if_not_exists(&self, conn: Self::Ref) -> Result<u64, C3p0Error> {
        conn.prep_exec(&self.create_table_sql_query, ())
            .map(|row| row.affected_rows())
            .map_err(into_c3p0_error)
    }

    fn drop_table_if_exists(&self, conn: Self::Ref) -> Result<u64, C3p0Error> {
        conn.prep_exec(&self.drop_table_sql_query, ())
            .map(|row| row.affected_rows())
            .map_err(into_c3p0_error)
    }

    fn count_all(&self, conn: Self::Ref) -> Result<i64, C3p0Error> {
        let mut stmt = conn
            .prepare(&self.count_all_sql_query)
            .map_err(into_c3p0_error)?;

        let result = stmt
            .first_exec(())
            .map_err(into_c3p0_error)?
            .ok_or_else(|| C3p0Error::IteratorError {
                message: "Cannot iterate next element".to_owned(),
            })?;
        Ok(result)
    }

    fn exists_by_id(&self, conn: Self::Ref, id: i64) -> Result<bool, C3p0Error> {
        let mut stmt = conn
            .prepare(&self.exists_by_id_sql_query)
            .map_err(into_c3p0_error)?;
        let result = stmt
            .first_exec(params! {
                "id" => id
            })
            .map_err(into_c3p0_error)?
            .ok_or_else(|| C3p0Error::IteratorError {
                message: "Cannot iterate next element".to_owned(),
            })?;
        Ok(result)
    }

    fn find_all(&self, conn: Self::Ref) -> Result<Vec<Model<DATA>>, C3p0Error> {
        conn.prep_exec(&self.find_all_sql_query, ())
            .map_err(into_c3p0_error)?
            .map(|row| self.to_model(row.map_err(into_c3p0_error)?))
            .collect()
    }

    fn find_by_id(&self, conn: Self::Ref, id: i64) -> Result<Option<Model<DATA>>, C3p0Error> {
        conn.prep_exec(
            &self.find_by_id_sql_query,
            params! {
                "id" => id
            },
        )
        .map_err(into_c3p0_error)?
        .next()
        .map(|row| self.to_model(row.map_err(into_c3p0_error)?))
        .transpose()
    }

    fn delete(&self, conn: Self::Ref, obj: &Model<DATA>) -> Result<u64, C3p0Error> {
        let mut stmt = conn
            .prepare(&self.delete_sql_query)
            .map_err(into_c3p0_error)?;
        let result = stmt
            .execute(params! {
                "id" => obj.id,
                "version" => obj.version,
            })
            .map(|result| result.affected_rows())
            .map_err(into_c3p0_error)?;

        if result == 0 {
            return Err(C3p0Error::OptimisticLockError{ message: format!("Cannot update data in table [{}] with id [{}], version [{}]: data was changed!",
                                                                        &self.qualified_table_name, &obj.id, &obj.version
            )});
        }

        Ok(result)
    }

    fn delete_all(&self, conn: Self::Ref) -> Result<u64, C3p0Error> {
        let mut stmt = conn
            .prepare(&self.delete_all_sql_query)
            .map_err(into_c3p0_error)?;
        stmt.execute(())
            .map(|result| result.affected_rows())
            .map_err(into_c3p0_error)
    }

    fn delete_by_id(&self, conn: Self::Ref, id: i64) -> Result<u64, C3p0Error> {
        let mut stmt = conn
            .prepare(&self.delete_by_id_sql_query)
            .map_err(into_c3p0_error)?;
        stmt.execute(params! {
            "id" => id
        })
        .map(|result| result.affected_rows())
        .map_err(into_c3p0_error)
    }

    fn save(&self, conn: Self::Ref, obj: NewModel<DATA>) -> Result<Model<DATA>, C3p0Error> {
        {
            let json_data = (self.codec.to_value)(&obj.data)?;
            let mut stmt = conn
                .prepare(&self.save_sql_query)
                .map_err(into_c3p0_error)?;
            stmt.execute(params! {
                "version" => &obj.version,
                "data" => &json_data
            })
            .map_err(into_c3p0_error)?;
        }

        let mut stmt = conn
            .prepare("SELECT LAST_INSERT_ID()")
            .map_err(into_c3p0_error)?;
        let id = stmt
            .execute(())
            .map_err(into_c3p0_error)?
            .next()
            .ok_or_else(|| C3p0Error::IteratorError {
                message: "Cannot iterate next element".to_owned(),
            })?
            .map_err(into_c3p0_error)
            .and_then(|row| get_or_error(&row, 0))?;

        Ok(Model {
            id,
            version: obj.version,
            data: obj.data,
        })
    }

    fn update(&self, conn: Self::Ref, obj: Model<DATA>) -> Result<Model<DATA>, C3p0Error> {
        let json_data = (self.codec.to_value)(&obj.data)?;

        let updated_model = Model {
            id: obj.id,
            version: obj.version + 1,
            data: obj.data,
        };

        let mut stmt = conn
            .prepare(&self.update_sql_query)
            .map_err(into_c3p0_error)?;

        let result = stmt
            .execute(params! {
                "updated_version" => updated_model.version,
                "data" => json_data,
                "id" => updated_model.id,
                "version" => obj.version,
            })
            .map(|result| result.affected_rows())
            .map_err(into_c3p0_error)?;

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
