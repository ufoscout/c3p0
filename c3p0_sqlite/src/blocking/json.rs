use c3p0_common::blocking::*;

use crate::blocking::rusqlite::{
    types::{FromSql, ToSql},
    Row, RowIndex
};
use crate::blocking::{SqliteC3p0Pool, SqliteConnection};
use c3p0_common::json::Queries;


pub trait SqliteC3p0JsonBuilder {
    fn build<DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send>(
        self,
    ) -> SqliteC3p0Json<DATA, DefaultJsonCodec>;
    fn build_with_codec<
        DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send,
        CODEC: JsonCodec<DATA>,
    >(
        self,
        codec: CODEC,
    ) -> SqliteC3p0Json<DATA, CODEC>;
}

impl SqliteC3p0JsonBuilder for C3p0JsonBuilder<SqliteC3p0Pool> {
    fn build<DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send>(
        self,
    ) -> SqliteC3p0Json<DATA, DefaultJsonCodec> {
        self.build_with_codec(DefaultJsonCodec {})
    }

    fn build_with_codec<
        DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send,
        CODEC: JsonCodec<DATA>,
    >(
        self,
        codec: CODEC,
    ) -> SqliteC3p0Json<DATA, CODEC> {
        let qualified_table_name = match &self.schema_name {
            Some(schema_name) => format!(r#"{}."{}""#, schema_name, self.table_name),
            None => self.table_name.clone(),
        };

        SqliteC3p0Json {
            phantom_data: std::marker::PhantomData,
            codec,
            queries: Queries {
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
                    {} integer primary key autoincrement,
                    {} integer not null,
                    {} JSON
                )
                "#,
                    qualified_table_name,
                    self.id_field_name,
                    self.version_field_name,
                    self.data_field_name
                ),

                drop_table_sql_query: format!("DROP TABLE IF EXISTS {}", qualified_table_name),
                drop_table_sql_query_cascade: format!(
                    "DROP TABLE IF EXISTS {}",
                    qualified_table_name
                ),

                lock_table_sql_query: None,

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
pub struct SqliteC3p0Json<DATA, CODEC: JsonCodec<DATA>>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send,
{
    phantom_data: std::marker::PhantomData<DATA>,

    codec: CODEC,
    queries: Queries,
}

impl<DATA, CODEC: JsonCodec<DATA>> SqliteC3p0Json<DATA, CODEC>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send,
{
    pub fn queries(&self) -> &Queries {
        &self.queries
    }

    #[inline]
    pub fn to_model(&self, row: &Row) -> Result<Model<DATA>, Box<dyn std::error::Error>> {
        to_model(&self.codec, row, 0, 1, 2)
    }

    /// Allows the execution of a custom sql query and returns the first entry in the result set.
    /// For this to work, the sql query:
    /// - must be a SELECT
    /// - must declare the ID, VERSION and DATA fields in this exact order
    pub fn fetch_one_optional_with_sql(
        &self,
        conn: &mut SqliteConnection,
        sql: &str,
        params: &[&dyn ToSql],
    ) -> Result<Option<Model<DATA>>, C3p0Error> {
        conn.fetch_one_optional(sql, params, |row| self.to_model(row))
    }

    /// Allows the execution of a custom sql query and returns the first entry in the result set.
    /// For this to work, the sql query:
    /// - must be a SELECT
    /// - must declare the ID, VERSION and DATA fields in this exact order
    pub fn fetch_one_with_sql(
        &self,
        conn: &mut SqliteConnection,
        sql: &str,
        params: &[&dyn ToSql],
    ) -> Result<Model<DATA>, C3p0Error> {
        conn.fetch_one(sql, params, |row| self.to_model(row))
    }

    /// Allows the execution of a custom sql query and returns all the entries in the result set.
    /// For this to work, the sql query:
    /// - must be a SELECT
    /// - must declare the ID, VERSION and DATA fields in this exact order
    pub fn fetch_all_with_sql(
        &self,
        conn: &mut SqliteConnection,
        sql: &str,
        params: &[&dyn ToSql],
    ) -> Result<Vec<Model<DATA>>, C3p0Error> {
        conn.fetch_all(sql, params, |row| self.to_model(row))
    }
}

impl<DATA, CODEC: JsonCodec<DATA>> C3p0Json<DATA, CODEC> for SqliteC3p0Json<DATA, CODEC>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send,
{
    type CONN = SqliteConnection;

    fn codec(&self) -> &CODEC {
        &self.codec
    }

    fn create_table_if_not_exists(&self, conn: &mut SqliteConnection) -> Result<(), C3p0Error> {
        conn.execute(&self.queries.create_table_sql_query, &[])?;
        Ok(())
    }

    fn drop_table_if_exists(
        &self,
        conn: &mut SqliteConnection,
        cascade: bool,
    ) -> Result<(), C3p0Error> {
        let query = if cascade {
            &self.queries.drop_table_sql_query_cascade
        } else {
            &self.queries.drop_table_sql_query
        };
        conn.execute(query, &[])?;
        Ok(())
    }

    fn count_all(&self, conn: &mut SqliteConnection) -> Result<u64, C3p0Error> {
        conn.fetch_one_value(&self.queries.count_all_sql_query, &[])
            .map(|val: i64| val as u64)
    }

    fn exists_by_id<'a, ID: Into<&'a IdType>>(
        &self,
        conn: &mut SqliteConnection,
        id: ID,
    ) -> Result<bool, C3p0Error> {
        conn.fetch_one_value(&self.queries.exists_by_id_sql_query, &[&id.into()])
    }

    fn fetch_all(&self, conn: &mut SqliteConnection) -> Result<Vec<Model<DATA>>, C3p0Error> {
        conn.fetch_all(&self.queries.find_all_sql_query, &[], |row| {
            self.to_model(row)
        })
    }

    fn fetch_all_for_update(
        &self,
        conn: &mut Self::CONN,
        _for_update: &ForUpdate,
    ) -> Result<Vec<Model<DATA>>, C3p0Error> {
        self.fetch_all(conn)
    }

    fn fetch_one_optional_by_id<'a, ID: Into<&'a IdType>>(
        &self,
        conn: &mut SqliteConnection,
        id: ID,
    ) -> Result<Option<Model<DATA>>, C3p0Error> {
        conn.fetch_one_optional(&self.queries.find_by_id_sql_query, &[&id.into()], |row| {
            self.to_model(row)
        })
    }

    fn fetch_one_optional_by_id_for_update<'a, ID: Into<&'a IdType>>(
        &'a self,
        conn: &mut Self::CONN,
        id: ID,
        _for_update: &ForUpdate,
    ) -> Result<Option<Model<DATA>>, C3p0Error> {
        self.fetch_one_optional_by_id(conn, id)
    }

    fn delete(
        &self,
        conn: &mut SqliteConnection,
        obj: Model<DATA>,
    ) -> Result<Model<DATA>, C3p0Error> {
        let result = conn.execute(&self.queries.delete_sql_query, &[&obj.id, &obj.version])?;

        if result == 0 {
            return Err(C3p0Error::OptimisticLockError{ message: format!("Cannot update data in table [{}] with id [{}], version [{}]: data was changed!",
                                                                        &self.queries.qualified_table_name, &obj.id, &obj.version
            )});
        }

        Ok(obj)
    }

    fn delete_all(&self, conn: &mut SqliteConnection) -> Result<u64, C3p0Error> {
        conn.execute(&self.queries.delete_all_sql_query, &[])
    }

    fn delete_by_id<'a, ID: Into<&'a IdType>>(
        &self,
        conn: &mut SqliteConnection,
        id: ID,
    ) -> Result<u64, C3p0Error> {
        conn.execute(&self.queries.delete_by_id_sql_query, &[id.into()])
    }

    fn save(
        &self,
        conn: &mut SqliteConnection,
        obj: NewModel<DATA>,
    ) -> Result<Model<DATA>, C3p0Error> {
        let json_data = self.codec.to_value(&obj.data)?;
        {
            conn.execute(&self.queries.save_sql_query, &[&obj.version, &json_data])?;
        }

        let id = { conn.fetch_one_value("SELECT last_insert_rowid()", &[])? };

        Ok(Model {
            id,
            version: obj.version,
            data: obj.data,
        })
    }

    fn update(
        &self,
        conn: &mut SqliteConnection,
        obj: Model<DATA>,
    ) -> Result<Model<DATA>, C3p0Error> {
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

#[inline]
pub fn to_model<
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send,
    CODEC: JsonCodec<DATA>,
    IdIdx: RowIndex,
    VersionIdx: RowIndex,
    DataIdx: RowIndex,
>(
    codec: &CODEC,
    row: &Row,
    id_index: IdIdx,
    version_index: VersionIdx,
    data_index: DataIdx,
) -> Result<Model<DATA>, Box<dyn std::error::Error>> {
    //id: Some(row.get(self.id_field_name.as_str())),
    //version: row.get(self.version_field_name.as_str()),
    //data: (conf.codec.from_value)(row.get(self.data_field_name.as_str()))?
    let id = get_or_error(&row, id_index)?;
    let version = get_or_error(&row, version_index)?;
    let data = codec.from_value(get_or_error(&row, data_index)?)?;
    Ok(Model { id, version, data })
}

#[inline]
fn get_or_error<I: RowIndex, T: FromSql>(row: &Row, index: I) -> Result<T, C3p0Error> {
    row.get(index).map_err(|err| C3p0Error::RowMapperError {
        cause: format!("Row contains no values. Err: {}", err),
    })
}
