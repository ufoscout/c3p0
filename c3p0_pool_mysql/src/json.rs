use crate::mysql::prelude::FromValue;
use crate::mysql::Row;
use crate::{MysqlConnection, MysqlPoolManager};
use c3p0_common::error::C3p0Error;
use c3p0_common::json::builder::C3p0JsonBuilder;
use c3p0_common::json::codec::DefaultJsonCodec;
use c3p0_common::json::{
    codec::JsonCodec,
    model::{IdType, Model, NewModel},
    C3p0Json, C3p0JsonManager, Queries,
};

pub trait MysqlJsonBuilder {
    fn build<DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned>(
        self,
    ) -> C3p0Json<DATA, DefaultJsonCodec, MysqlJsonManager<DATA, DefaultJsonCodec>>;
    fn build_with_codec<
        DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
        CODEC: JsonCodec<DATA>,
    >(
        self,
        codec: CODEC,
    ) -> C3p0Json<DATA, CODEC, MysqlJsonManager<DATA, CODEC>>;
}

impl MysqlJsonBuilder for C3p0JsonBuilder<MysqlPoolManager> {
    fn build<DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned>(
        self,
    ) -> C3p0Json<DATA, DefaultJsonCodec, MysqlJsonManager<DATA, DefaultJsonCodec>> {
        self.build_with_codec(DefaultJsonCodec {})
    }

    fn build_with_codec<
        DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
        CODEC: JsonCodec<DATA>,
    >(
        self,
        codec: CODEC,
    ) -> C3p0Json<DATA, CODEC, MysqlJsonManager<DATA, CODEC>> {
        let qualified_table_name = match &self.schema_name {
            Some(schema_name) => format!(r#"{}."{}""#, schema_name, self.table_name),
            None => self.table_name.clone(),
        };

        let json_manager = MysqlJsonManager {
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

                lock_table_sql_query: Some(format!("LOCK TABLES {} WRITE", qualified_table_name)),

                qualified_table_name,
                table_name: self.table_name,
                id_field_name: self.id_field_name,
                version_field_name: self.version_field_name,
                data_field_name: self.data_field_name,
                schema_name: self.schema_name,
            },
        };

        C3p0Json::new(json_manager)
    }
}

#[derive(Clone)]
pub struct MysqlJsonManager<DATA, CODEC: JsonCodec<DATA>>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
{
    phantom_data: std::marker::PhantomData<DATA>,

    codec: CODEC,
    queries: Queries,
}

impl<DATA, CODEC: JsonCodec<DATA>> MysqlJsonManager<DATA, CODEC>
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

impl<DATA, CODEC: JsonCodec<DATA>> C3p0JsonManager<DATA, CODEC> for MysqlJsonManager<DATA, CODEC>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
{
    type CONNECTION = MysqlConnection;

    fn codec(&self) -> &CODEC {
        &self.codec
    }

    fn queries(&self) -> &Queries {
        &self.queries
    }

    fn create_table_if_not_exists(&self, conn: &MysqlConnection) -> Result<(), C3p0Error> {
        conn.execute(&self.queries.create_table_sql_query, &[])?;
        Ok(())
    }

    fn drop_table_if_exists(&self, conn: &MysqlConnection) -> Result<(), C3p0Error> {
        conn.execute(&self.queries.drop_table_sql_query, &[])?;
        Ok(())
    }

    fn count_all(&self, conn: &MysqlConnection) -> Result<i64, C3p0Error> {
        conn.fetch_one_value(&self.queries.count_all_sql_query, &[])
    }

    fn exists_by_id<'a, ID: Into<&'a IdType>>(
        &self,
        conn: &MysqlConnection,
        id: ID,
    ) -> Result<bool, C3p0Error> {
        conn.fetch_one_value(&self.queries.exists_by_id_sql_query, &[&id.into()])
    }

    fn find_all(&self, conn: &MysqlConnection) -> Result<Vec<Model<DATA>>, C3p0Error> {
        conn.fetch_all(&self.queries.find_all_sql_query, &[], |row| {
            Ok(self.to_model(row)?)
        })
    }

    fn find_by_id<'a, ID: Into<&'a IdType>>(
        &self,
        conn: &MysqlConnection,
        id: ID,
    ) -> Result<Option<Model<DATA>>, C3p0Error> {
        conn.fetch_one_option(&self.queries.find_by_id_sql_query, &[&id.into()], |row| {
            Ok(self.to_model(row)?)
        })
    }

    fn delete(&self, conn: &MysqlConnection, obj: &Model<DATA>) -> Result<u64, C3p0Error> {
        let result = conn.execute(&self.queries.delete_sql_query, &[&obj.id, &obj.version])?;

        if result == 0 {
            return Err(C3p0Error::OptimisticLockError{ message: format!("Cannot update data in table [{}] with id [{}], version [{}]: data was changed!",
                                                                        &self.queries.qualified_table_name, &obj.id, &obj.version
            )});
        }

        Ok(result)
    }

    fn delete_all(&self, conn: &MysqlConnection) -> Result<u64, C3p0Error> {
        conn.execute(&self.queries.delete_all_sql_query, &[])
    }

    fn delete_by_id<'a, ID: Into<&'a IdType>>(
        &self,
        conn: &MysqlConnection,
        id: ID,
    ) -> Result<u64, C3p0Error> {
        conn.execute(&self.queries.delete_by_id_sql_query, &[id.into()])
    }

    fn save(&self, conn: &MysqlConnection, obj: NewModel<DATA>) -> Result<Model<DATA>, C3p0Error> {
        let json_data = self.codec.to_value(&obj.data)?;
        {
            conn.execute(&self.queries.save_sql_query, &[&obj.version, &json_data])?;
        }

        let id = { conn.fetch_one_value("SELECT LAST_INSERT_ID()", &[])? };

        Ok(Model {
            id,
            version: obj.version,
            data: obj.data,
        })
    }

    fn update(&self, conn: &MysqlConnection, obj: Model<DATA>) -> Result<Model<DATA>, C3p0Error> {
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

fn get_or_error<T: FromValue>(row: &Row, index: usize) -> Result<T, C3p0Error> {
    row.get(index).ok_or_else(|| C3p0Error::SqlError {
        cause: format!("Row contains no values for index {}", index),
    })
}
