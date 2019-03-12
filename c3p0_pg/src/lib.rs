use postgres::error::Error;
use postgres::rows::Row;
use postgres::Connection;

#[derive(Clone)]
pub struct Model<DATA>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
{
    pub id: Option<i64>,
    pub version: i32,
    pub data: DATA,
}

impl<DATA> Model<DATA>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
{
    pub fn new(data: DATA) -> Self {
        Model {
            id: None,
            version: 0,
            data,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Config {
    pub id_field_name: String,
    pub version_field_name: String,
    pub data_field_name: String,
    pub table_name: String,
    pub schema_name: Option<String>,
    pub qualified_table_name: String,

    pub find_all_sql_query: String,
    pub find_by_id_sql_query: String,
    pub save_sql_query: String,
    pub create_table_sql_query: String,
    pub drop_table_sql_query: String,
}

#[derive(Clone, Debug)]
pub struct ConfigBuilder {
    id_field_name: String,
    version_field_name: String,
    data_field_name: String,
    table_name: String,
    schema_name: Option<String>,
}

impl ConfigBuilder {
    pub fn new<T: Into<String>>(table_name: T) -> Self {
        let table_name = table_name.into();
        ConfigBuilder {
            table_name: table_name.clone(),
            id_field_name: "id".to_owned(),
            version_field_name: "version".to_owned(),
            data_field_name: "data".to_owned(),
            schema_name: None,
        }
    }

    pub fn with_id_field_name<T: Into<String>>(mut self, id_field_name: T) -> ConfigBuilder {
        self.id_field_name = id_field_name.into();
        self
    }

    pub fn with_version_field_name<T: Into<String>>(
        mut self,
        version_field_name: T,
    ) -> ConfigBuilder {
        self.version_field_name = version_field_name.into();
        self
    }

    pub fn with_data_field_name<T: Into<String>>(mut self, data_field_name: T) -> ConfigBuilder {
        self.data_field_name = data_field_name.into();
        self
    }

    pub fn with_schema_name<T: Into<Option<String>>>(mut self, schema_name: T) -> ConfigBuilder {
        self.schema_name = schema_name.into();
        self
    }

    pub fn build(self) -> Config {
        let qualified_table_name = match &self.schema_name {
            Some(schema_name) => format!(r#"{}."{}""#, schema_name, self.table_name),
            None => self.table_name.clone(),
        };

        Config {
            find_all_sql_query: format!(
                "SELECT {}, {}, {} FROM {} ORDER BY {} ASC",
                self.id_field_name,
                self.version_field_name,
                self.data_field_name,
                qualified_table_name,
                self.id_field_name,
            ),

            find_by_id_sql_query: format!(
                "SELECT {}, {}, {} FROM {} WHERE {} = $1",
                self.id_field_name,
                self.version_field_name,
                self.data_field_name,
                qualified_table_name,
                self.id_field_name,
            ),

            save_sql_query: format!(
                "INSERT INTO {} ({}, {}) VALUES ($1, $2) RETURNING {}",
                qualified_table_name,
                self.version_field_name,
                self.data_field_name,
                self.id_field_name
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

            drop_table_sql_query: format!("DROP TABLE IF EXISTS {}",
                qualified_table_name
            ),

            qualified_table_name,
            table_name: self.table_name,
            id_field_name: self.id_field_name,
            version_field_name: self.version_field_name,
            data_field_name: self.data_field_name,
            schema_name: self.schema_name,
        }
    }
}

pub trait JpoPg<DATA>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
{
    fn conf(&self) -> &Config;

    fn to_model(&self, row: Row) -> Model<DATA> {
        //id: Some(row.get(self.id_field_name.as_str())),
        //version: row.get(self.version_field_name.as_str()),
        //data: serde_json::from_value::<DATA>(row.get(self.data_field_name.as_str())).unwrap()
        let id = Some(row.get(0));
        let version = row.get(1);
        let data = serde_json::from_value::<DATA>(row.get(2)).unwrap();
        Model { id, version, data }
    }

    fn create_table_if_not_exists(&self, conn: &Connection) -> Result<u64, Error> {
        conn.execute(&self.conf().create_table_sql_query, &[])
    }

    fn drop_table_if_exists(&self, conn: &Connection) -> Result<u64, Error> {
        conn.execute(&self.conf().drop_table_sql_query, &[])
    }

    fn find_all(&self, conn: &Connection) -> Result<Vec<Model<DATA>>, Error> {
        let conf = self.conf();
        let stmt = conn.prepare(&conf.find_all_sql_query)?;
        let result = stmt.query(&[])?
            .iter()
            .map(|row| self.to_model(row))
            .collect();
        Ok(result)
    }

    fn find_by_id(&self, conn: &Connection, id: i64) -> Result<Option<Model<DATA>>, Error> {
        let conf = self.conf();
        let stmt = conn.prepare(&conf.find_by_id_sql_query)?;
        let result = stmt.query(&[&id])?
            .iter()
            .next()
            .map(|row| self.to_model(row));
        Ok(result)
    }

    fn save(&self, conn: &Connection, obj: Model<DATA>) -> Result<Model<DATA>, Error> {
        let conf = self.conf();
        let stmt = conn.prepare(&conf.save_sql_query)?;
        let json_data = serde_json::to_value(&obj.data).expect("Cannot serialize obj to Value");
        let id: i64 = stmt
            .query(&[&obj.version, &json_data])?
            .iter()
            .next()
            .expect("Cannot iterate next element")
            .get(0);

        Ok(Model {
            id: Some(id),
            version: obj.version,
            data: obj.data,
        })
    }
}

#[derive(Clone)]
pub struct SimpleRepository<DATA>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
{
    conf: Config,
    phantom_data: std::marker::PhantomData<DATA>,
}

impl<DATA> SimpleRepository<DATA>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
{
    pub fn build(conf: Config) -> Self {
        SimpleRepository {
            conf,
            phantom_data: std::marker::PhantomData,
        }
    }
}

impl<DATA> JpoPg<DATA> for SimpleRepository<DATA>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
{
    fn conf(&self) -> &Config {
        &self.conf
    }
}
