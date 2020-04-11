use crate::blocking::postgres::{row::Row, types::ToSql};
use crate::blocking::*;
use c3p0_common::blocking::*;
use c3p0_common::json::Queries;

pub trait PgC3p0JsonBuilder {
    fn build<DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send>(
        self,
    ) -> PgC3p0Json<DATA, DefaultJsonCodec>;
    fn build_with_codec<
        DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send,
        CODEC: JsonCodec<DATA>,
    >(
        self,
        codec: CODEC,
    ) -> PgC3p0Json<DATA, CODEC>;
}

impl PgC3p0JsonBuilder for C3p0JsonBuilder<PgC3p0Pool> {
    fn build<DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send>(
        self,
    ) -> PgC3p0Json<DATA, DefaultJsonCodec> {
        self.build_with_codec(DefaultJsonCodec {})
    }

    fn build_with_codec<
        DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send,
        CODEC: JsonCodec<DATA>,
    >(
        self,
        codec: CODEC,
    ) -> PgC3p0Json<DATA, CODEC> {
        PgC3p0Json {
            phantom_data: std::marker::PhantomData,
            codec,
            queries: build_pg_queries(self),
        }
    }
}

#[derive(Clone)]
pub struct PgC3p0Json<DATA, CODEC: JsonCodec<DATA>>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send,
{
    phantom_data: std::marker::PhantomData<DATA>,

    codec: CODEC,
    queries: Queries,
}

impl<DATA, CODEC: JsonCodec<DATA>> PgC3p0Json<DATA, CODEC>
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
        conn: &mut PgConnection,
        sql: &str,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<Option<Model<DATA>>, C3p0Error> {
        conn.fetch_one_optional(sql, params, |row| self.to_model(row))
    }

    /// Allows the execution of a custom sql query and returns the first entry in the result set.
    /// For this to work, the sql query:
    /// - must be a SELECT
    /// - must declare the ID, VERSION and DATA fields in this exact order
    pub fn fetch_one_with_sql(
        &self,
        conn: &mut PgConnection,
        sql: &str,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<Model<DATA>, C3p0Error> {
        conn.fetch_one(sql, params, |row| self.to_model(row))
    }

    /// Allows the execution of a custom sql query and returns all the entries in the result set.
    /// For this to work, the sql query:
    /// - must be a SELECT
    /// - must declare the ID, VERSION and DATA fields in this exact order
    pub fn fetch_all_with_sql(
        &self,
        conn: &mut PgConnection,
        sql: &str,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<Vec<Model<DATA>>, C3p0Error> {
        conn.fetch_all(sql, params, |row| self.to_model(row))
    }
}

impl<DATA, CODEC: JsonCodec<DATA>> C3p0Json<DATA, CODEC> for PgC3p0Json<DATA, CODEC>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send,
{
    type Conn = PgConnection;

    fn codec(&self) -> &CODEC {
        &self.codec
    }

    fn create_table_if_not_exists(&self, conn: &mut PgConnection) -> Result<(), C3p0Error> {
        conn.execute(&self.queries.create_table_sql_query, &[])?;
        Ok(())
    }

    fn drop_table_if_exists(
        &self,
        conn: &mut PgConnection,
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

    fn count_all(&self, conn: &mut PgConnection) -> Result<u64, C3p0Error> {
        conn.fetch_one_value(&self.queries.count_all_sql_query, &[])
            .map(|val: i64| val as u64)
    }

    fn exists_by_id<'a, ID: Into<&'a IdType>>(
        &self,
        conn: &mut PgConnection,
        id: ID,
    ) -> Result<bool, C3p0Error> {
        conn.fetch_one_value(&self.queries.exists_by_id_sql_query, &[&id.into()])
    }

    fn fetch_all(&self, conn: &mut PgConnection) -> Result<Vec<Model<DATA>>, C3p0Error> {
        conn.fetch_all(&self.queries.find_all_sql_query, &[], |row| {
            self.to_model(row)
        })
    }

    fn fetch_all_for_update(
        &self,
        conn: &mut Self::Conn,
        for_update: &ForUpdate,
    ) -> Result<Vec<Model<DATA>>, C3p0Error> {
        let sql = format!(
            "{}\n{}",
            &self.queries.find_all_sql_query,
            for_update.to_sql()
        );
        conn.fetch_all(&sql, &[], |row| self.to_model(row))
    }

    fn fetch_one_optional_by_id<'a, ID: Into<&'a IdType>>(
        &self,
        conn: &mut PgConnection,
        id: ID,
    ) -> Result<Option<Model<DATA>>, C3p0Error> {
        conn.fetch_one_optional(&self.queries.find_by_id_sql_query, &[&id.into()], |row| {
            self.to_model(row)
        })
    }

    fn fetch_one_optional_by_id_for_update<'a, ID: Into<&'a IdType>>(
        &'a self,
        conn: &mut Self::Conn,
        id: ID,
        for_update: &ForUpdate,
    ) -> Result<Option<Model<DATA>>, C3p0Error> {
        let sql = format!(
            "{}\n{}",
            &self.queries.find_by_id_sql_query,
            for_update.to_sql()
        );
        conn.fetch_one_optional(&sql, &[&id.into()], |row| self.to_model(row))
    }

    fn delete(&self, conn: &mut PgConnection, obj: Model<DATA>) -> Result<Model<DATA>, C3p0Error> {
        let result = conn.execute(&self.queries.delete_sql_query, &[&obj.id, &obj.version])?;

        if result == 0 {
            return Err(C3p0Error::OptimisticLockError{ message: format!("Cannot update data in table [{}] with id [{}], version [{}]: data was changed!",
                                                                        &self.queries.qualified_table_name, &obj.id, &obj.version
            )});
        }

        Ok(obj)
    }

    fn delete_all(&self, conn: &mut PgConnection) -> Result<u64, C3p0Error> {
        conn.execute(&self.queries.delete_all_sql_query, &[])
    }

    fn delete_by_id<'a, ID: Into<&'a IdType>>(
        &self,
        conn: &mut PgConnection,
        id: ID,
    ) -> Result<u64, C3p0Error> {
        conn.execute(&self.queries.delete_by_id_sql_query, &[id.into()])
    }

    fn save(&self, conn: &mut PgConnection, obj: NewModel<DATA>) -> Result<Model<DATA>, C3p0Error> {
        let json_data = self.codec().to_value(&obj.data)?;
        let id = conn.fetch_one_value(&self.queries.save_sql_query, &[&obj.version, &json_data])?;
        Ok(Model {
            id,
            version: obj.version,
            data: obj.data,
        })
    }

    fn update(&self, conn: &mut PgConnection, obj: Model<DATA>) -> Result<Model<DATA>, C3p0Error> {
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
