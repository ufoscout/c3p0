use crate::error::C3p0Error;
use crate::json::codec::DefaultJsonCodec;
use crate::json::{codec::JsonCodec, JsonManagerBase, Model};
use crate::types::OptString;
use rusqlite::Row;

#[derive(Clone)]
pub struct SqliteJsonManager<'a, DATA, CODEC: JsonCodec<DATA>>
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
    pub lock_table_exclusively_sql_query: String,
}

#[derive(Clone)]
pub struct SqliteJsonManagerBuilder<DATA, CODEC: JsonCodec<DATA>>
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

impl<DATA> SqliteJsonManagerBuilder<DATA, DefaultJsonCodec>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
{
    pub fn new<T: Into<String>>(table_name: T) -> Self {
        let table_name = table_name.into();
        SqliteJsonManagerBuilder {
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

impl<DATA, CODEC: JsonCodec<DATA>> SqliteJsonManagerBuilder<DATA, CODEC>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
{
    pub fn with_codec<NEWCODEC: JsonCodec<DATA>>(
        self,
        codec: NEWCODEC,
    ) -> SqliteJsonManagerBuilder<DATA, NEWCODEC> {
        SqliteJsonManagerBuilder {
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
    ) -> SqliteJsonManagerBuilder<DATA, CODEC> {
        self.id_field_name = id_field_name.into();
        self
    }

    pub fn with_version_field_name<T: Into<String>>(
        mut self,
        version_field_name: T,
    ) -> SqliteJsonManagerBuilder<DATA, CODEC> {
        self.version_field_name = version_field_name.into();
        self
    }

    pub fn with_data_field_name<T: Into<String>>(
        mut self,
        data_field_name: T,
    ) -> SqliteJsonManagerBuilder<DATA, CODEC> {
        self.data_field_name = data_field_name.into();
        self
    }

    pub fn with_schema_name<O: Into<OptString>>(
        mut self,
        schema_name: O,
    ) -> SqliteJsonManagerBuilder<DATA, CODEC> {
        self.schema_name = schema_name.into().value;
        self
    }

    pub fn build<'a>(self) -> SqliteJsonManager<'a, DATA, CODEC> {
        let qualified_table_name = match &self.schema_name {
            Some(schema_name) => format!(r#"{}."{}""#, schema_name, self.table_name),
            None => self.table_name.clone(),
        };

        SqliteJsonManager {
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

            lock_table_exclusively_sql_query: format!(
                "LOCK TABLE {} IN ACCESS EXCLUSIVE MODE",
                qualified_table_name
            ),

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

impl<'a, DATA, CODEC: JsonCodec<DATA>> JsonManagerBase<DATA, CODEC>
    for SqliteJsonManager<'a, DATA, CODEC>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
{
    type Conn = crate::client::sqlite::pool::SqliteConnection<'a>;

    fn codec(&self) -> &CODEC {
        &self.codec
    }

    fn to_model(&self, row: &Row) -> Result<Model<DATA>, C3p0Error> {
        //id: Some(row.get(self.id_field_name.as_str())),
        //version: row.get(self.version_field_name.as_str()),
        //data: (conf.codec.from_value)(row.get(self.data_field_name.as_str()))?
        let id = get_or_error(&row, 0)?;
        let version = get_or_error(&row, 1)?;
        let data = self.codec.from_value(get_or_error(&row, 2)?)?;
        Ok(Model { id, version, data })
    }

    fn id_field_name(&self) -> &str {
        &self.id_field_name
    }

    fn version_field_name(&self) -> &str {
        &self.version_field_name
    }

    fn data_field_name(&self) -> &str {
        &self.data_field_name
    }

    fn table_name(&self) -> &str {
        &self.table_name
    }

    fn schema_name(&self) -> &Option<String> {
        &self.schema_name
    }

    fn qualified_table_name(&self) -> &str {
        &self.qualified_table_name
    }

    fn count_all_sql_query(&self) -> &str {
        &self.count_all_sql_query
    }

    fn exists_by_id_sql_query(&self) -> &str {
        &self.exists_by_id_sql_query
    }

    fn find_all_sql_query(&self) -> &str {
        &self.find_all_sql_query
    }

    fn find_by_id_sql_query(&self) -> &str {
        &self.find_by_id_sql_query
    }

    fn delete_sql_query(&self) -> &str {
        &self.delete_sql_query
    }

    fn delete_all_sql_query(&self) -> &str {
        &self.delete_all_sql_query
    }

    fn delete_by_id_sql_query(&self) -> &str {
        &self.delete_by_id_sql_query
    }

    fn save_sql_query(&self) -> &str {
        &self.save_sql_query
    }

    fn update_sql_query(&self) -> &str {
        &self.update_sql_query
    }

    fn create_table_sql_query(&self) -> &str {
        &self.create_table_sql_query
    }

    fn drop_table_sql_query(&self) -> &str {
        &self.drop_table_sql_query
    }

    fn lock_table_exclusively_sql_query(&self) -> &str {
        &self.lock_table_exclusively_sql_query
    }
}

fn get_or_error<T: rusqlite::types::FromSql>(row: &Row, index: usize) -> Result<T, C3p0Error> {
    row.get(index).map_err(|err| C3p0Error::SqlError {
        cause: format!("Row contains no values for index {}. Err: {}", index, err),
    })
}
