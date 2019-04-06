use crate::codec::Codec;
use crate::error::C3p0Error;
use mysql::{params, Conn, Row};
use serde::Deserialize;
use serde_derive::{Deserialize, Serialize};

pub mod codec;
pub mod error;

type IdType = i64;
type VersionType = i32;

#[derive(Clone, Serialize, Deserialize)]
pub struct Model<DATA>
where
    DATA: Clone + serde::ser::Serialize,
{
    pub id: IdType,
    pub version: VersionType,
    #[serde(bound(deserialize = "DATA: Deserialize<'de>"))]
    pub data: DATA,
}

impl<DATA> Model<DATA>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
{
    pub fn into_new(self) -> NewModel<DATA> {
        NewModel {
            version: 0,
            data: self.data,
        }
    }
}

impl<'a, DATA> Into<&'a IdType> for &'a Model<DATA>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
{
    fn into(self) -> &'a IdType {
        &self.id
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct NewModel<DATA>
where
    DATA: Clone + serde::ser::Serialize,
{
    pub version: VersionType,
    #[serde(bound(deserialize = "DATA: Deserialize<'de>"))]
    pub data: DATA,
}

impl<DATA> NewModel<DATA>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
{
    pub fn new(data: DATA) -> Self {
        NewModel { version: 0, data }
    }
}

#[derive(Clone)]
pub struct Config<DATA>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
{
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

    pub delete_all_sql_query: String,
    pub delete_by_id_sql_query: String,

    pub save_sql_query: String,

    pub create_table_sql_query: String,
    pub drop_table_sql_query: String,
}

#[derive(Clone)]
pub struct ConfigBuilder<DATA>
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

impl<DATA> ConfigBuilder<DATA>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
{
    pub fn new<T: Into<String>>(table_name: T) -> Self {
        let table_name = table_name.into();
        ConfigBuilder {
            codec: Default::default(),
            table_name: table_name.clone(),
            id_field_name: "id".to_owned(),
            version_field_name: "version".to_owned(),
            data_field_name: "data".to_owned(),
            schema_name: None,
        }
    }

    pub fn with_codec(mut self, codec: Codec<DATA>) -> ConfigBuilder<DATA> {
        self.codec = codec;
        self
    }

    pub fn with_id_field_name<T: Into<String>>(mut self, id_field_name: T) -> ConfigBuilder<DATA> {
        self.id_field_name = id_field_name.into();
        self
    }

    pub fn with_version_field_name<T: Into<String>>(
        mut self,
        version_field_name: T,
    ) -> ConfigBuilder<DATA> {
        self.version_field_name = version_field_name.into();
        self
    }

    pub fn with_data_field_name<T: Into<String>>(
        mut self,
        data_field_name: T,
    ) -> ConfigBuilder<DATA> {
        self.data_field_name = data_field_name.into();
        self
    }

    pub fn with_schema_name<O: Into<OptString>>(mut self, schema_name: O) -> ConfigBuilder<DATA> {
        self.schema_name = schema_name.into().value;
        self
    }

    pub fn build(self) -> Config<DATA> {
        let qualified_table_name = match &self.schema_name {
            Some(schema_name) => format!(r#"{}."{}""#, schema_name, self.table_name),
            None => self.table_name.clone(),
        };

        Config {
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

            delete_all_sql_query: format!("DELETE FROM {}", qualified_table_name,),

            delete_by_id_sql_query: format!(
                "DELETE FROM {} WHERE {} = :id",
                qualified_table_name, self.id_field_name,
            ),

            save_sql_query: format!(
                "INSERT INTO {} ({}, {}) VALUES (:version, :data)",
                qualified_table_name, self.version_field_name, self.data_field_name
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

pub struct OptString {
    value: Option<String>,
}

impl Into<OptString> for String {
    fn into(self) -> OptString {
        OptString { value: Some(self) }
    }
}

impl Into<OptString> for &str {
    fn into(self) -> OptString {
        OptString {
            value: Some(self.to_owned()),
        }
    }
}

impl Into<OptString> for Option<String> {
    fn into(self) -> OptString {
        OptString { value: self }
    }
}

impl Into<OptString> for Option<&str> {
    fn into(self) -> OptString {
        OptString {
            value: self.map(|val| val.to_owned()),
        }
    }
}

pub trait C3p0<DATA>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
{
    fn conf(&self) -> &Config<DATA>;

    fn to_model(&self, row: Row) -> Result<Model<DATA>, C3p0Error> {
        //id: Some(row.get(self.id_field_name.as_str())),
        //version: row.get(self.version_field_name.as_str()),
        //data: (conf.codec.from_value)(row.get(self.data_field_name.as_str()))?
        let conf = self.conf();
        let id = row.get(0).unwrap();
        let version = row.get(1).unwrap();
        let data = (conf.codec.from_value)(row.get(2).unwrap())?;
        Ok(Model { id, version, data })
    }

    fn create_table_if_not_exists(&self, conn: &mut Conn) -> Result<u64, C3p0Error> {
        conn.prep_exec(&self.conf().create_table_sql_query, ())
            .map(|row| row.affected_rows())
            .map_err(C3p0Error::from)
    }

    fn drop_table_if_exists(&self, conn: &mut Conn) -> Result<u64, C3p0Error> {
        conn.prep_exec(&self.conf().drop_table_sql_query, ())
            .map(|row| row.affected_rows())
            .map_err(C3p0Error::from)
    }

    fn count_all(&self, conn: &mut Conn) -> Result<IdType, C3p0Error> {
        let conf = self.conf();
        let mut stmt = conn.prepare(&conf.count_all_sql_query)?;
        let result = stmt
            .execute(())?
            .into_iter()
            .next()
            .ok_or_else(|| C3p0Error::IteratorError {
                message: "Cannot iterate next element".to_owned(),
            })?
            .map(|row| row.get(0).unwrap())
            .unwrap();
        Ok(result)
    }

    fn exists_by_id<'a, ID: Into<&'a IdType>>(
        &'a self,
        conn: &mut Conn,
        id: ID,
    ) -> Result<bool, C3p0Error> {
        let conf = self.conf();
        let mut stmt = conn.prepare(&conf.exists_by_id_sql_query)?;
        let id_into = id.into();
        let result = stmt
            .execute(params! {
                "id" => id_into
            })?
            .into_iter()
            .next()
            .ok_or_else(|| C3p0Error::IteratorError {
                message: "Cannot iterate next element".to_owned(),
            })?
            .map(|row| row.get(0).unwrap())
            .unwrap();
        Ok(result)
    }

    fn find_all(&self, conn: &mut Conn) -> Result<Vec<Model<DATA>>, C3p0Error> {
        let conf = self.conf();
        conn.prep_exec(&conf.find_all_sql_query, ())?
            .into_iter()
            .map(|row| self.to_model(row.unwrap()))
            .collect()
    }

    fn find_by_id<'a, ID: Into<&'a IdType>>(
        &'a self,
        conn: &mut Conn,
        id: ID,
    ) -> Result<Option<Model<DATA>>, C3p0Error> {
        let conf = self.conf();
        let id_into = id.into();
        conn.prep_exec(
            &conf.find_by_id_sql_query,
            params! {
                "id" => id_into
            },
        )?
        .into_iter()
        .next()
        .map(|row| self.to_model(row.unwrap()))
        .transpose()
    }

    fn delete_all(&self, conn: &mut Conn) -> Result<u64, C3p0Error> {
        let conf = self.conf();
        let mut stmt = conn.prepare(&conf.delete_all_sql_query)?;
        stmt.execute(())
            .map(|result| result.affected_rows())
            .map_err(C3p0Error::from)
    }

    fn delete_by_id<'a, ID: Into<&'a IdType>>(
        &'a self,
        conn: &mut Conn,
        id: ID,
    ) -> Result<u64, C3p0Error> {
        let conf = self.conf();
        let mut stmt = conn.prepare(&conf.delete_by_id_sql_query)?;
        stmt.execute(params! {
            "id" => id.into()
        })
        .map(|result| result.affected_rows())
        .map_err(C3p0Error::from)
    }

    fn save(&self, conn: &mut Conn, obj: NewModel<DATA>) -> Result<Model<DATA>, C3p0Error> {
        let conf = self.conf();
        {
            let json_data = (conf.codec.to_value)(&obj.data)?;
            let mut stmt = conn.prepare(&conf.save_sql_query)?;
            stmt.execute(params! {
                "version" => &obj.version,
                "data" => &json_data
            })?;
        }

        let mut stmt = conn.prepare("SELECT LAST_INSERT_ID()")?;
        let id = stmt
            .execute(())?
            .into_iter()
            .next()
            .ok_or_else(|| C3p0Error::IteratorError {
                message: "Cannot iterate next element".to_owned(),
            })?
            .map(|row| row.get(0).unwrap())
            .unwrap();

        Ok(Model {
            id,
            version: obj.version,
            data: obj.data,
        })
    }
}

#[derive(Clone)]
pub struct C3p0Repository<DATA>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
{
    conf: Config<DATA>,
    phantom_data: std::marker::PhantomData<DATA>,
}

impl<DATA> C3p0Repository<DATA>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
{
    pub fn build(conf: Config<DATA>) -> Self {
        C3p0Repository {
            conf,
            phantom_data: std::marker::PhantomData,
        }
    }
}

impl<DATA> C3p0<DATA> for C3p0Repository<DATA>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
{
    fn conf(&self) -> &Config<DATA> {
        &self.conf
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use serde_json;

    #[test]
    fn model_should_be_serializable() -> Result<(), Box<std::error::Error>> {
        let model = Model {
            id: 1,
            version: 1,
            data: SimpleData {
                name: "test".to_owned(),
            },
        };

        let serialize = serde_json::to_string(&model)?;
        let deserialize: Model<SimpleData> = serde_json::from_str(&serialize)?;

        assert_eq!(model.id, deserialize.id);
        assert_eq!(model.version, deserialize.version);
        assert_eq!(model.data, deserialize.data);

        Ok(())
    }

    #[test]
    fn new_model_should_be_serializable() -> Result<(), Box<std::error::Error>> {
        let model = NewModel::new(SimpleData {
            name: "test".to_owned(),
        });

        let serialize = serde_json::to_string(&model)?;
        let deserialize: NewModel<SimpleData> = serde_json::from_str(&serialize)?;

        assert_eq!(model.version, deserialize.version);
        assert_eq!(model.data, deserialize.data);
        Ok(())
    }

    #[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
    struct SimpleData {
        name: String,
    }
}
