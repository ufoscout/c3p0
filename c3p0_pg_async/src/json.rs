use crate::pg_async::driver::{
    row::{Row, RowIndex},
    types::{FromSql, ToSql},
};
use crate::pool::{PgC3p0PoolAsync, PgConnectionAsync};
use async_trait::async_trait;
use c3p0_common::error::C3p0Error;
use c3p0_common::json::builder::C3p0JsonBuilder;
use c3p0_common::json::codec::DefaultJsonCodec;
use c3p0_common::json::{
    codec::JsonCodec,
    model::{IdType, Model, NewModel},
    Queries,
};
use c3p0_common::sql::ForUpdate;
use c3p0_common_async::C3p0JsonAsync;
use serde::export::fmt::Display;

pub trait PgC3p0JsonAsyncBuilder {
    fn build<DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned>(
        self,
    ) -> PgC3p0JsonAsync<DATA, DefaultJsonCodec>;
    fn build_with_codec<
        DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
        CODEC: JsonCodec<DATA>,
    >(
        self,
        codec: CODEC,
    ) -> PgC3p0JsonAsync<DATA, CODEC>;
}

impl PgC3p0JsonAsyncBuilder for C3p0JsonBuilder<PgC3p0PoolAsync> {
    fn build<DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned>(
        self,
    ) -> PgC3p0JsonAsync<DATA, DefaultJsonCodec> {
        self.build_with_codec(DefaultJsonCodec {})
    }

    fn build_with_codec<
        DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
        CODEC: JsonCodec<DATA>,
    >(
        self,
        codec: CODEC,
    ) -> PgC3p0JsonAsync<DATA, CODEC> {
        let qualified_table_name = match &self.schema_name {
            Some(schema_name) => format!(r#"{}."{}""#, schema_name, self.table_name),
            None => self.table_name.clone(),
        };

        PgC3p0JsonAsync {
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
                drop_table_sql_query_cascade: format!(
                    "DROP TABLE IF EXISTS {} CASCADE",
                    qualified_table_name
                ),

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
pub struct PgC3p0JsonAsync<DATA, CODEC: JsonCodec<DATA>>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
{
    phantom_data: std::marker::PhantomData<DATA>,

    codec: CODEC,
    queries: Queries,
}

impl<DATA, CODEC: JsonCodec<DATA>> PgC3p0JsonAsync<DATA, CODEC>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
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
    pub async fn fetch_one_optional_with_sql(
        &self,
        conn: &mut PgConnectionAsync,
        sql: &str,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<Option<Model<DATA>>, C3p0Error> {
        conn.fetch_one_optional(sql, params, |row| self.to_model(row))
            .await
    }

    /// Allows the execution of a custom sql query and returns the first entry in the result set.
    /// For this to work, the sql query:
    /// - must be a SELECT
    /// - must declare the ID, VERSION and DATA fields in this exact order
    pub async fn fetch_one_with_sql(
        &self,
        conn: &mut PgConnectionAsync,
        sql: &str,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<Model<DATA>, C3p0Error> {
        conn.fetch_one(sql, params, |row| self.to_model(row)).await
    }

    /// Allows the execution of a custom sql query and returns all the entries in the result set.
    /// For this to work, the sql query:
    /// - must be a SELECT
    /// - must declare the ID, VERSION and DATA fields in this exact order
    pub async fn fetch_all_with_sql(
        &self,
        conn: &mut PgConnectionAsync,
        sql: &str,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<Vec<Model<DATA>>, C3p0Error> {
        conn.fetch_all(sql, params, |row| self.to_model(row)).await
    }
}

#[async_trait(?Send)]
impl<DATA, CODEC: JsonCodec<DATA>> C3p0JsonAsync<DATA, CODEC> for PgC3p0JsonAsync<DATA, CODEC>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
{
    type CONN = PgConnectionAsync;

    fn codec(&self) -> &CODEC {
        &self.codec
    }

    async fn create_table_if_not_exists(
        &self,
        conn: &mut PgConnectionAsync,
    ) -> Result<(), C3p0Error> {
        conn.execute(&self.queries.create_table_sql_query, &[])
            .await?;
        Ok(())
    }

    async fn drop_table_if_exists(
        &self,
        conn: &mut PgConnectionAsync,
        cascade: bool,
    ) -> Result<(), C3p0Error> {
        let query = if cascade {
            &self.queries.drop_table_sql_query_cascade
        } else {
            &self.queries.drop_table_sql_query
        };
        conn.execute(query, &[]).await?;
        Ok(())
    }

    async fn count_all(&self, conn: &mut PgConnectionAsync) -> Result<u64, C3p0Error> {
        conn.fetch_one_value(&self.queries.count_all_sql_query, &[])
            .await
            .map(|val: i64| val as u64)
    }

    async fn exists_by_id<'a, ID: Into<&'a IdType>>(
        &'a self,
        conn: &mut PgConnectionAsync,
        id: ID,
    ) -> Result<bool, C3p0Error> {
        conn.fetch_one_value(&self.queries.exists_by_id_sql_query, &[&id.into()])
            .await
    }

    async fn fetch_all(&self, conn: &mut PgConnectionAsync) -> Result<Vec<Model<DATA>>, C3p0Error> {
        conn.fetch_all(&self.queries.find_all_sql_query, &[], |row| {
            self.to_model(row)
        })
        .await
    }

    async fn fetch_all_for_update(
        &self,
        conn: &mut PgConnectionAsync,
        for_update: &ForUpdate,
    ) -> Result<Vec<Model<DATA>>, C3p0Error> {
        let sql = format!(
            "{}\n{}",
            &self.queries.find_all_sql_query,
            for_update.to_sql()
        );
        conn.fetch_all(&sql, &[], |row| self.to_model(row)).await
    }

    async fn fetch_one_optional_by_id<'a, ID: Into<&'a IdType>>(
        &'a self,
        conn: &mut PgConnectionAsync,
        id: ID,
    ) -> Result<Option<Model<DATA>>, C3p0Error> {
        conn.fetch_one_optional(&self.queries.find_by_id_sql_query, &[&id.into()], |row| {
            self.to_model(row)
        })
        .await
    }

    async fn fetch_one_optional_by_id_for_update<'a, ID: Into<&'a IdType>>(
        &'a self,
        conn: &mut PgConnectionAsync,
        id: ID,
        for_update: &ForUpdate,
    ) -> Result<Option<Model<DATA>>, C3p0Error> {
        let sql = format!(
            "{}\n{}",
            &self.queries.find_by_id_sql_query,
            for_update.to_sql()
        );
        conn.fetch_one_optional(&sql, &[&id.into()], |row| self.to_model(row))
            .await
    }

    async fn fetch_one_by_id<'a, ID: Into<&'a IdType>>(
        &'a self,
        conn: &mut PgConnectionAsync,
        id: ID,
    ) -> Result<Model<DATA>, C3p0Error> {
        self.fetch_one_optional_by_id(conn, id)
            .await
            .and_then(|result| result.ok_or_else(|| C3p0Error::ResultNotFoundError))
    }

    async fn fetch_one_by_id_for_update<'a, ID: Into<&'a IdType>>(
        &'a self,
        conn: &mut PgConnectionAsync,
        id: ID,
        for_update: &ForUpdate,
    ) -> Result<Model<DATA>, C3p0Error> {
        self.fetch_one_optional_by_id_for_update(conn, id, for_update)
            .await
            .and_then(|result| result.ok_or_else(|| C3p0Error::ResultNotFoundError))
    }

    async fn delete(
        &self,
        conn: &mut PgConnectionAsync,
        obj: Model<DATA>,
    ) -> Result<Model<DATA>, C3p0Error> {
        let result = conn
            .execute(&self.queries.delete_sql_query, &[&obj.id, &obj.version])
            .await?;

        if result == 0 {
            return Err(C3p0Error::OptimisticLockError{ message: format!("Cannot update data in table [{}] with id [{}], version [{}]: data was changed!",
                                                                        &self.queries.qualified_table_name, &obj.id, &obj.version
            )});
        }

        Ok(obj)
    }

    async fn delete_all(&self, conn: &mut PgConnectionAsync) -> Result<u64, C3p0Error> {
        conn.execute(&self.queries.delete_all_sql_query, &[]).await
    }

    async fn delete_by_id<'a, ID: Into<&'a IdType>>(
        &'a self,
        conn: &mut PgConnectionAsync,
        id: ID,
    ) -> Result<u64, C3p0Error> {
        conn.execute(&self.queries.delete_by_id_sql_query, &[id.into()])
            .await
    }

    async fn save(
        &self,
        conn: &mut PgConnectionAsync,
        obj: NewModel<DATA>,
    ) -> Result<Model<DATA>, C3p0Error> {
        let json_data = self.codec().to_value(&obj.data)?;
        let id = conn
            .fetch_one_value(&self.queries.save_sql_query, &[&obj.version, &json_data])
            .await?;
        Ok(Model {
            id,
            version: obj.version,
            data: obj.data,
        })
    }

    async fn update(
        &self,
        conn: &mut PgConnectionAsync,
        obj: Model<DATA>,
    ) -> Result<Model<DATA>, C3p0Error> {
        let json_data = self.codec().to_value(&obj.data)?;

        let updated_model = Model {
            id: obj.id,
            version: obj.version + 1,
            data: obj.data,
        };

        let result = conn
            .execute(
                &self.queries.update_sql_query,
                &[
                    &updated_model.version,
                    &json_data,
                    &updated_model.id,
                    &obj.version,
                ],
            )
            .await?;

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
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
    CODEC: JsonCodec<DATA>,
    IdIdx: RowIndex + Display,
    VersionIdx: RowIndex + Display,
    DataIdx: RowIndex + Display,
>(
    codec: &CODEC,
    row: &Row,
    id_index: IdIdx,
    version_index: VersionIdx,
    data_index: DataIdx,
) -> Result<Model<DATA>, Box<dyn std::error::Error>> {
    let id = get_or_error(&row, id_index)?;
    let version = get_or_error(&row, version_index)?;
    let data = codec.from_value(get_or_error(&row, data_index)?)?;
    Ok(Model { id, version, data })
}

#[inline]
pub fn get_or_error<'a, I: RowIndex + Display, T: FromSql<'a>>(
    row: &'a Row,
    index: I,
) -> Result<T, C3p0Error> {
    row.try_get(&index)
        .map_err(|err| C3p0Error::RowMapperError {
            cause: format!("Row contains no values for index {}. Err: {}", index, err),
        })
}
