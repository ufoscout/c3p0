use crate::error::into_c3p0_error;
use crate::pg::driver::{
    rows::Row,
    types::{FromSql, ToSql},
};
use crate::pool::{PgC3p0Pool, PgConnection};
use c3p0_common::error::C3p0Error;
use c3p0_common::json::builder::C3p0JsonBuilder;
use c3p0_common::json::codec::DefaultJsonCodec;
use c3p0_common::json::{
    codec::JsonCodec,
    model::{IdType, Model, NewModel},
    C3p0Json, Queries,
};

pub trait PgC3p0JsonBuilder {
    fn build<DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned>(
        self,
    ) -> PgC3p0Json<DATA, DefaultJsonCodec>;
    fn build_with_codec<
        DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
        CODEC: JsonCodec<DATA>,
    >(
        self,
        codec: CODEC,
    ) -> PgC3p0Json<DATA, CODEC>;
}

impl PgC3p0JsonBuilder for C3p0JsonBuilder<PgC3p0Pool> {
    fn build<DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned>(
        self,
    ) -> PgC3p0Json<DATA, DefaultJsonCodec> {
        self.build_with_codec(DefaultJsonCodec {})
    }

    fn build_with_codec<
        DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
        CODEC: JsonCodec<DATA>,
    >(
        self,
        codec: CODEC,
    ) -> PgC3p0Json<DATA, CODEC> {
        let qualified_table_name = match &self.schema_name {
            Some(schema_name) => format!(r#"{}."{}""#, schema_name, self.table_name),
            None => self.table_name.clone(),
        };

        PgC3p0Json {
            phantom_data: std::marker::PhantomData,
            codec,
            queries: Queries {
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
                drop_table_sql_query_cascade: format!("DROP TABLE IF EXISTS {} CASCADE", qualified_table_name),

                lock_table_sql_query: Some(format!(
                    "LOCK TABLE {} IN ACCESS EXCLUSIVE MODE",
                    qualified_table_name
                )),

                qualified_table_name,
                table_name: self.table_name,
                id_field_name: self.id_field_name,
                version_field_name: self.version_field_name,
                data_field_name: self.data_field_name,
                schema_name: self.schema_name,
            },
        }
    }
}

#[derive(Clone)]
pub struct PgC3p0Json<DATA, CODEC: JsonCodec<DATA>>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
{
    phantom_data: std::marker::PhantomData<DATA>,

    codec: CODEC,
    queries: Queries,
}

impl<DATA, CODEC: JsonCodec<DATA>> PgC3p0Json<DATA, CODEC>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
{
    pub fn queries(&self) -> &Queries {
        &self.queries
    }

    pub fn to_model(&self, row: &Row) -> Result<Model<DATA>, Box<dyn std::error::Error>> {
        //id: Some(row.get(self.id_field_name.as_str())),
        //version: row.get(self.version_field_name.as_str()),
        //data: (conf.codec.from_value)(row.get(self.data_field_name.as_str()))?
        let id = get_or_error(&row, 0)?;
        let version = get_or_error(&row, 1)?;
        let data = self.codec.from_value(get_or_error(&row, 2)?)?;
        Ok(Model { id, version, data })
    }

    /// Allows the execution of a custom sql query and returns the first entry in the result set.
    /// For this to work, the sql query:
    /// - must be a SELECT
    /// - must declare the ID, VERSION and DATA fields in this exact order
    pub fn fetch_one_with_sql(
        &self,
        conn: &PgConnection,
        sql: &str,
        params: &[&dyn ToSql],
    ) -> Result<Option<Model<DATA>>, C3p0Error> {
        conn.fetch_one_option(sql, params, |row| self.to_model(row))
    }

    /// Allows the execution of a custom sql query and returns all the entries in the result set.
    /// For this to work, the sql query:
    /// - must be a SELECT
    /// - must declare the ID, VERSION and DATA fields in this exact order
    pub fn fetch_all_with_sql(
        &self,
        conn: &PgConnection,
        sql: &str,
        params: &[&dyn ToSql],
    ) -> Result<Vec<Model<DATA>>, C3p0Error> {
        conn.fetch_all(sql, params, |row| self.to_model(row))
    }
}

impl<DATA, CODEC: JsonCodec<DATA>> C3p0Json<DATA, CODEC> for PgC3p0Json<DATA, CODEC>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
{
    type CONN = PgConnection;

    fn codec(&self) -> &CODEC {
        &self.codec
    }

    fn create_table_if_not_exists(&self, conn: &PgConnection) -> Result<(), C3p0Error> {
        conn.execute(&self.queries.create_table_sql_query, &[])?;
        Ok(())
    }

    fn drop_table_if_exists(&self, conn: &PgConnection, cascade: bool) -> Result<(), C3p0Error> {
        let query = if cascade {
            &self.queries.drop_table_sql_query_cascade
        } else {
            &self.queries.drop_table_sql_query
        };
        conn.execute(query, &[])?;
        Ok(())
    }

    fn count_all(&self, conn: &PgConnection) -> Result<i64, C3p0Error> {
        conn.fetch_one_value(&self.queries.count_all_sql_query, &[])
    }

    fn exists_by_id<'a, ID: Into<&'a IdType>>(
        &self,
        conn: &PgConnection,
        id: ID,
    ) -> Result<bool, C3p0Error> {
        conn.fetch_one_value(&self.queries.exists_by_id_sql_query, &[&id.into()])
    }

    fn fetch_all(&self, conn: &PgConnection) -> Result<Vec<Model<DATA>>, C3p0Error> {
        conn.fetch_all(&self.queries.find_all_sql_query, &[], |row| {
            self.to_model(row)
        })
    }

    fn fetch_one_by_id<'a, ID: Into<&'a IdType>>(
        &self,
        conn: &PgConnection,
        id: ID,
    ) -> Result<Option<Model<DATA>>, C3p0Error> {
        conn.fetch_one_option(&self.queries.find_by_id_sql_query, &[&id.into()], |row| {
            self.to_model(row)
        })
    }

    fn delete(&self, conn: &PgConnection, obj: &Model<DATA>) -> Result<u64, C3p0Error> {
        let result = conn.execute(&self.queries.delete_sql_query, &[&obj.id, &obj.version])?;

        if result == 0 {
            return Err(C3p0Error::OptimisticLockError{ message: format!("Cannot update data in table [{}] with id [{}], version [{}]: data was changed!",
                                                                        &self.queries.qualified_table_name, &obj.id, &obj.version
            )});
        }

        Ok(result)
    }

    fn delete_all(&self, conn: &PgConnection) -> Result<u64, C3p0Error> {
        conn.execute(&self.queries.delete_all_sql_query, &[])
    }

    fn delete_by_id<'a, ID: Into<&'a IdType>>(
        &self,
        conn: &PgConnection,
        id: ID,
    ) -> Result<u64, C3p0Error> {
        conn.execute(&self.queries.delete_by_id_sql_query, &[id.into()])
    }

    fn save(&self, conn: &PgConnection, obj: NewModel<DATA>) -> Result<Model<DATA>, C3p0Error> {
        let json_data = self.codec().to_value(&obj.data)?;
        let id = conn.fetch_one_value(&self.queries.save_sql_query, &[&obj.version, &json_data])?;
        Ok(Model {
            id,
            version: obj.version,
            data: obj.data,
        })
    }

    fn update(&self, conn: &PgConnection, obj: Model<DATA>) -> Result<Model<DATA>, C3p0Error> {
        let json_data = self.codec().to_value(&obj.data)?;

        let updated_model = Model {
            id: obj.id,
            version: obj.version + 1,
            data: obj.data,
        };

        let result = conn.execute(
            &self.queries.update_sql_query,
            &[
                &updated_model.version,
                &json_data,
                &updated_model.id,
                &obj.version,
            ],
        )?;

        if result == 0 {
            return Err(C3p0Error::OptimisticLockError{ message: format!("Cannot update data in table [{}] with id [{}], version [{}]: data was changed!",
                                                                        &self.queries.qualified_table_name, &updated_model.id, &obj.version
            )});
        }

        Ok(updated_model)
    }
}

fn get_or_error<T: FromSql>(row: &Row, index: usize) -> Result<T, C3p0Error> {
    row.get_opt(index)
        .ok_or_else(|| C3p0Error::SqlError {
            cause: format!("Row contains no values for index {}", index),
        })?
        .map_err(into_c3p0_error)
}
