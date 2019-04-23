use super::error::into_c3p0_error;
use crate::error::C3p0Error;
use crate::json::codec::DefaultJsonCodec;
use crate::json::{codec::JsonCodec, JsonManager, Model, NewModel};
use crate::types::OptString;
use postgres::rows::Row;

#[derive(Clone)]
pub struct PostgresJsonManager<'a, DATA, CODEC: JsonCodec<DATA>>
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
pub struct PostgresJsonManagerBuilder<DATA, CODEC: JsonCodec<DATA>>
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

impl<DATA> PostgresJsonManagerBuilder<DATA, DefaultJsonCodec>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
{
    pub fn new<T: Into<String>>(table_name: T) -> Self {
        let table_name = table_name.into();
        PostgresJsonManagerBuilder {
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

impl<DATA, CODEC: JsonCodec<DATA>> PostgresJsonManagerBuilder<DATA, CODEC>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
{
    pub fn with_codec<NEWCODEC: JsonCodec<DATA>>(
        self,
        codec: NEWCODEC,
    ) -> PostgresJsonManagerBuilder<DATA, NEWCODEC> {
        PostgresJsonManagerBuilder {
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
    ) -> PostgresJsonManagerBuilder<DATA, CODEC> {
        self.id_field_name = id_field_name.into();
        self
    }

    pub fn with_version_field_name<T: Into<String>>(
        mut self,
        version_field_name: T,
    ) -> PostgresJsonManagerBuilder<DATA, CODEC> {
        self.version_field_name = version_field_name.into();
        self
    }

    pub fn with_data_field_name<T: Into<String>>(
        mut self,
        data_field_name: T,
    ) -> PostgresJsonManagerBuilder<DATA, CODEC> {
        self.data_field_name = data_field_name.into();
        self
    }

    pub fn with_schema_name<O: Into<OptString>>(
        mut self,
        schema_name: O,
    ) -> PostgresJsonManagerBuilder<DATA, CODEC> {
        self.schema_name = schema_name.into().value;
        self
    }

    pub fn build<'a>(self) -> PostgresJsonManager<'a, DATA, CODEC> {
        let qualified_table_name = match &self.schema_name {
            Some(schema_name) => format!(r#"{}."{}""#, schema_name, self.table_name),
            None => self.table_name.clone(),
        };

        PostgresJsonManager {
            phantom_a: std::marker::PhantomData,
            phantom_data: std::marker::PhantomData,

            count_all_sql_query: format!("SELECT COUNT(*) FROM {}", qualified_table_name,),

            exists_by_id_sql_query: format!(
                "SELECT EXISTS (SELECT 1 FROM {} WHERE {} = $1)",
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
                "SELECT {}, {}, {} FROM {} WHERE {} = $1 LIMIT 1",
                self.id_field_name,
                self.version_field_name,
                self.data_field_name,
                qualified_table_name,
                self.id_field_name,
            ),

            delete_sql_query: format!(
                "DELETE FROM {} WHERE {} = $1 AND {} = $2",
                qualified_table_name, self.id_field_name, self.version_field_name,
            ),

            delete_all_sql_query: format!("DELETE FROM {}", qualified_table_name,),

            delete_by_id_sql_query: format!(
                "DELETE FROM {} WHERE {} = $1",
                qualified_table_name, self.id_field_name,
            ),

            save_sql_query: format!(
                "INSERT INTO {} ({}, {}) VALUES ($1, $2) RETURNING {}",
                qualified_table_name,
                self.version_field_name,
                self.data_field_name,
                self.id_field_name
            ),

            update_sql_query: format!(
                "UPDATE {} SET {} = $1, {} = $2 WHERE {} = $3 AND {} = $4",
                qualified_table_name,
                self.version_field_name,
                self.data_field_name,
                self.id_field_name,
                self.version_field_name,
            ),

            create_table_sql_query: format!(
                r#"
                CREATE TABLE IF NOT EXISTS {} (
                    {} bigserial primary key,
                    {} int not null,
                    {} JSONB
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

impl<'a, DATA, CODEC: JsonCodec<DATA>> PostgresJsonManager<'a, DATA, CODEC>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
{
    fn to_model(&self, row: Row) -> Result<Model<DATA>, C3p0Error> {
        //id: Some(row.get(self.id_field_name.as_str())),
        //version: row.get(self.version_field_name.as_str()),
        //data: (conf.codec.from_value)(row.get(self.data_field_name.as_str()))?
        let id = get_or_error(&row, 0)?;
        let version = get_or_error(&row, 1)?;
        let data = self.codec.from_value(get_or_error(&row, 2)?)?;
        Ok(Model { id, version, data })
    }
}

impl<'a, DATA, CODEC: JsonCodec<DATA>> JsonManager<DATA> for PostgresJsonManager<'a, DATA, CODEC>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
{
    type Conn = postgres::Connection;
    type Ref = &'a Self::Conn;

    fn create_table_if_not_exists(&self, conn: Self::Ref) -> Result<u64, C3p0Error> {
        conn.execute(&self.create_table_sql_query, &[])
            .map_err(into_c3p0_error)
    }

    fn drop_table_if_exists(&self, conn: Self::Ref) -> Result<u64, C3p0Error> {
        conn.execute(&self.drop_table_sql_query, &[])
            .map_err(into_c3p0_error)
    }

    fn count_all(&self, conn: Self::Ref) -> Result<i64, C3p0Error> {
        let stmt = conn
            .prepare(&self.count_all_sql_query)
            .map_err(into_c3p0_error)?;
        let result = stmt
            .query(&[])
            .map_err(into_c3p0_error)?
            .iter()
            .next()
            .ok_or_else(|| C3p0Error::IteratorError {
                message: "Cannot iterate next element".to_owned(),
            })?
            .get(0);
        Ok(result)
    }

    fn exists_by_id(&self, conn: Self::Ref, id: i64) -> Result<bool, C3p0Error> {
        let stmt = conn
            .prepare(&self.exists_by_id_sql_query)
            .map_err(into_c3p0_error)?;
        let result = stmt
            .query(&[&id])
            .map_err(into_c3p0_error)?
            .iter()
            .next()
            .ok_or_else(|| C3p0Error::IteratorError {
                message: "Cannot iterate next element".to_owned(),
            })?
            .get(0);
        Ok(result)
    }

    fn find_all(&self, conn: Self::Ref) -> Result<Vec<Model<DATA>>, C3p0Error> {
        let stmt = conn
            .prepare(&self.find_all_sql_query)
            .map_err(into_c3p0_error)?;
        stmt.query(&[])
            .map_err(into_c3p0_error)?
            .iter()
            .map(|row| self.to_model(row))
            .collect()
    }

    fn find_by_id(&self, conn: Self::Ref, id: i64) -> Result<Option<Model<DATA>>, C3p0Error> {
        let stmt = conn
            .prepare(&self.find_by_id_sql_query)
            .map_err(into_c3p0_error)?;
        stmt.query(&[&id])
            .map_err(into_c3p0_error)?
            .iter()
            .next()
            .map(|row| self.to_model(row))
            .transpose()
    }

    fn delete(&self, conn: Self::Ref, obj: &Model<DATA>) -> Result<u64, C3p0Error> {
        let stmt = conn
            .prepare(&self.delete_sql_query)
            .map_err(into_c3p0_error)?;
        let result = stmt
            .execute(&[&obj.id, &obj.version])
            .map_err(into_c3p0_error)?;

        if result == 0 {
            return Err(C3p0Error::OptimisticLockError{ message: format!("Cannot update data in table [{}] with id [{}], version [{}]: data was changed!",
                                                                        &self.qualified_table_name, &obj.id, &obj.version
            )});
        }

        Ok(result)
    }

    fn delete_all(&self, conn: Self::Ref) -> Result<u64, C3p0Error> {
        let stmt = conn
            .prepare(&self.delete_all_sql_query)
            .map_err(into_c3p0_error)?;
        stmt.execute(&[]).map_err(into_c3p0_error)
    }

    fn delete_by_id(&self, conn: Self::Ref, id: i64) -> Result<u64, C3p0Error> {
        let stmt = conn
            .prepare(&self.delete_by_id_sql_query)
            .map_err(into_c3p0_error)?;
        stmt.execute(&[&id]).map_err(into_c3p0_error)
    }

    fn save(&self, conn: Self::Ref, obj: NewModel<DATA>) -> Result<Model<DATA>, C3p0Error> {
        let json_data = self.codec.to_value(&obj.data)?;
        let stmt = conn
            .prepare(&self.save_sql_query)
            .map_err(into_c3p0_error)?;
        let id = stmt
            .query(&[&obj.version, &json_data])
            .map_err(into_c3p0_error)?
            .iter()
            .next()
            .ok_or_else(|| C3p0Error::IteratorError {
                message: "Cannot iterate next element".to_owned(),
            })?
            .get(0);

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

        let stmt = conn
            .prepare(&self.update_sql_query)
            .map_err(into_c3p0_error)?;
        let result = stmt
            .execute(&[
                &updated_model.version,
                &json_data,
                &updated_model.id,
                &obj.version,
            ])
            .map_err(into_c3p0_error)?;

        if result == 0 {
            return Err(C3p0Error::OptimisticLockError{ message: format!("Cannot update data in table [{}] with id [{}], version [{}]: data was changed!",
                &self.qualified_table_name, &updated_model.id, &obj.version
            )});
        }

        Ok(updated_model)
    }
}

fn get_or_error<T: postgres::types::FromSql>(
    row: &Row,
    index: usize,
) -> Result<T, C3p0Error> {
    row.get_opt(index)
        .ok_or_else(|| C3p0Error::SqlError {
            cause: format!("Row contains no values for index {}", index),
        })?
        .map_err(into_c3p0_error)
}
