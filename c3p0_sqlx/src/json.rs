use crate::{into_c3p0_error, Db, DbRow, SqlxConnection};
use async_trait::async_trait;
use c3p0_common::json::Queries;
use c3p0_common::*;
use sqlx::{Row, IntoArguments};
use futures_util::TryStreamExt;
use futures::stream::Collect;
use std::iter::Iterator;
use sqlx::query::Query;

pub trait SqlxC3p0JsonBuilder {
    fn build<DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send + Sync>(
        self,
    ) -> SqlxC3p0Json<DATA, DefaultJsonCodec>;
    fn build_with_codec<
        DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send + Sync,
        CODEC: JsonCodec<DATA>,
    >(
        self,
        codec: CODEC,
    ) -> SqlxC3p0Json<DATA, CODEC>;
}

impl SqlxC3p0JsonBuilder for C3p0JsonBuilder<crate::SqlxC3p0Pool> {
    fn build<DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send + Sync>(
        self,
    ) -> SqlxC3p0Json<DATA, DefaultJsonCodec> {
        self.build_with_codec(DefaultJsonCodec {})
    }

    fn build_with_codec<
        DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send + Sync,
        CODEC: JsonCodec<DATA>,
    >(
        self,
        codec: CODEC,
    ) -> SqlxC3p0Json<DATA, CODEC> {
        SqlxC3p0Json {
            phantom_data: std::marker::PhantomData,
            codec,
            queries: crate::common::build_pg_queries(self),
        }
    }
}

#[derive(Clone)]
pub struct SqlxC3p0Json<DATA, CODEC: JsonCodec<DATA>>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send + Sync,
{
    phantom_data: std::marker::PhantomData<DATA>,

    codec: CODEC,
    queries: Queries,
}

impl<DATA, CODEC: JsonCodec<DATA>> SqlxC3p0Json<DATA, CODEC>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send + Sync,
{
    pub fn queries(&self) -> &Queries {
        &self.queries
    }

    #[inline]
    pub fn to_model(&self, row: &DbRow) -> Result<Model<DATA>, C3p0Error> {
        to_model(&self.codec, row, 0, 1, 2)
    }

    /// Allows the execution of a custom sql query and returns the first entry in the result set.
    /// For this to work, the sql query:
    /// - must be a SELECT
    /// - must declare the ID, VERSION and DATA fields in this exact order
    pub async fn fetch_one_optional_with_sql<'a, A: 'a + Send + IntoArguments<'a, Db>>(
        &self,
        conn: &mut SqlxConnection,
        sql: Query<'a, Db, A>,
    ) -> Result<Option<Model<DATA>>, C3p0Error> {
        sql.fetch_optional(conn.get_conn())
            .await
            .map_err(into_c3p0_error)?
            .map(|row| self.to_model(&row) ).transpose()
    }

       /// Allows the execution of a custom sql query and returns the first entry in the result set.
       /// For this to work, the sql query:
       /// - must be a SELECT
       /// - must declare the ID, VERSION and DATA fields in this exact order
       pub async fn fetch_one_with_sql<'a, A: 'a + Send + IntoArguments<'a, Db>>(
           &self,
           conn: &mut SqlxConnection,
           sql: Query<'a, Db, A>,
       ) -> Result<Model<DATA>, C3p0Error> {
           sql.fetch_one(conn.get_conn())
               .await
               .map_err(into_c3p0_error)
               .and_then(|row| {
                   self.to_model(&row)
               })
       }


       /// Allows the execution of a custom sql query and returns all the entries in the result set.
       /// For this to work, the sql query:
       /// - must be a SELECT
       /// - must declare the ID, VERSION and DATA fields in this exact order
       pub async fn fetch_all_with_sql<'a, A: 'a + Send + IntoArguments<'a, Db>>(
           &self,
           conn: &mut SqlxConnection,
           sql: Query<'a, Db, A>,
       ) -> Result<Vec<Model<DATA>>, C3p0Error> {
           sql.fetch_all(conn.get_conn())
               .await
               .map_err(into_c3p0_error)?
               .iter()
               .map(|row| {
                   self.to_model(&row)
               })
               .collect::<Result<Vec<_>, C3p0Error>>()
       }

}

#[async_trait]
impl<DATA, CODEC: JsonCodec<DATA>> C3p0Json<DATA, CODEC> for SqlxC3p0Json<DATA, CODEC>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send + Sync,
{
    type Conn = SqlxConnection;

    fn codec(&self) -> &CODEC {
        &self.codec
    }

    async fn create_table_if_not_exists(&self, conn: &mut Self::Conn) -> Result<(), C3p0Error> {
        sqlx::query(&self.queries.create_table_sql_query)
            .execute(conn.get_conn())
            .await
            .map_err(into_c3p0_error)
            .map(|_| ())
    }

    async fn drop_table_if_exists(
        &self,
        conn: &mut Self::Conn,
        cascade: bool,
    ) -> Result<(), C3p0Error> {
        let query = if cascade {
            &self.queries.drop_table_sql_query_cascade
        } else {
            &self.queries.drop_table_sql_query
        };
        sqlx::query(query)
            .execute(conn.get_conn())
            .await
            .map_err(into_c3p0_error)
            .map(|_| ())
    }

    async fn count_all(&self, conn: &mut Self::Conn) -> Result<u64, C3p0Error> {
        sqlx::query(&self.queries.count_all_sql_query)
            .fetch_one(conn.get_conn())
            .await
            .and_then(|row| row.try_get(0))
            .map_err(into_c3p0_error)
            .map(|val: i64| val as u64)
    }

    async fn exists_by_id<'a, ID: Into<&'a IdType> + Send>(
        &'a self,
        conn: &mut Self::Conn,
        id: ID,
    ) -> Result<bool, C3p0Error> {
        sqlx::query(&self.queries.exists_by_id_sql_query)
            .bind(id.into())
            .fetch_one(conn.get_conn())
            .await
            .and_then(|row| row.try_get(0))
            .map_err(into_c3p0_error)
    }

    async fn fetch_all(&self, conn: &mut Self::Conn) -> Result<Vec<Model<DATA>>, C3p0Error> {
        self.fetch_all_with_sql(conn, sqlx::query(&self.queries.find_all_sql_query)).await
    }

    async fn fetch_all_for_update(
        &self,
        conn: &mut Self::Conn,
        for_update: &ForUpdate,
    ) -> Result<Vec<Model<DATA>>, C3p0Error> {
        let sql = format!(
            "{}\n{}",
            &self.queries.find_all_sql_query,
            for_update.to_sql()
        );
        self.fetch_all_with_sql(conn, sqlx::query(&sql)).await
    }

    async fn fetch_one_optional_by_id<'a, ID: Into<&'a IdType> + Send>(
        &'a self,
        conn: &mut Self::Conn,
        id: ID,
    ) -> Result<Option<Model<DATA>>, C3p0Error> {
        self.fetch_one_optional_with_sql(conn, sqlx::query(&self.queries.find_by_id_sql_query).bind(id.into())).await
    }

    async fn fetch_one_optional_by_id_for_update<'a, ID: Into<&'a IdType> + Send>(
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
        self.fetch_one_optional_with_sql(conn, sqlx::query(&sql).bind(id.into())).await
    }

    async fn fetch_one_by_id<'a, ID: Into<&'a IdType> + Send>(
        &'a self,
        conn: &mut Self::Conn,
        id: ID,
    ) -> Result<Model<DATA>, C3p0Error> {
        self.fetch_one_with_sql(conn, sqlx::query(&self.queries.find_by_id_sql_query).bind(id.into())).await
        // self.fetch_one_optional_by_id(conn, id)
        //     .await
        //     .and_then(|result| result.ok_or_else(|| C3p0Error::ResultNotFoundError))
    }

    async fn fetch_one_by_id_for_update<'a, ID: Into<&'a IdType> + Send>(
        &'a self,
        conn: &mut Self::Conn,
        id: ID,
        for_update: &ForUpdate,
    ) -> Result<Model<DATA>, C3p0Error> {
        let sql = format!(
            "{}\n{}",
            &self.queries.find_by_id_sql_query,
            for_update.to_sql()
        );
        self.fetch_one_with_sql(conn, sqlx::query(&sql).bind(id.into())).await

        // self.fetch_one_optional_by_id_for_update(conn, id, for_update)
        //     .await
        //     .and_then(|result| result.ok_or_else(|| C3p0Error::ResultNotFoundError))
    }

    async fn delete(
        &self,
        conn: &mut Self::Conn,
        obj: Model<DATA>,
    ) -> Result<Model<DATA>, C3p0Error> {
        unimplemented!()
        // let result = conn
        //     .execute(&self.queries.delete_sql_query, &[&obj.id, &obj.version])
        //     .await?;
        //
        // if result == 0 {
        //     return Err(C3p0Error::OptimisticLockError{ message: format!("Cannot update data in table [{}] with id [{}], version [{}]: data was changed!",
        //                                                                 &self.queries.qualified_table_name, &obj.id, &obj.version
        //     )});
        // }
        //
        // Ok(obj)
    }

    async fn delete_all(&self, conn: &mut Self::Conn) -> Result<u64, C3p0Error> {
        unimplemented!()
        // conn.execute(&self.queries.delete_all_sql_query, &[]).await
    }

    async fn delete_by_id<'a, ID: Into<&'a IdType> + Send>(
        &'a self,
        conn: &mut Self::Conn,
        id: ID,
    ) -> Result<u64, C3p0Error> {
        unimplemented!()
        // conn.execute(&self.queries.delete_by_id_sql_query, &[id.into()])
        //     .await
    }

    async fn save(
        &self,
        conn: &mut Self::Conn,
        obj: NewModel<DATA>,
    ) -> Result<Model<DATA>, C3p0Error> {
        unimplemented!()
        // let json_data = self.codec().to_value(&obj.data)?;
        // let id = conn
        //     .fetch_one_value(&self.queries.save_sql_query, &[&obj.version, &json_data])
        //     .await?;
        // Ok(Model {
        //     id,
        //     version: obj.version,
        //     data: obj.data,
        // })
    }

    async fn update(
        &self,
        conn: &mut Self::Conn,
        obj: Model<DATA>,
    ) -> Result<Model<DATA>, C3p0Error> {
        unimplemented!()
        // let json_data = self.codec().to_value(&obj.data)?;
        //
        // let updated_model = Model {
        //     id: obj.id,
        //     version: obj.version + 1,
        //     data: obj.data,
        // };
        //
        // let result = conn
        //     .execute(
        //         &self.queries.update_sql_query,
        //         &[
        //             &updated_model.version,
        //             &json_data,
        //             &updated_model.id,
        //             &obj.version,
        //         ],
        //     )
        //     .await?;
        //
        // if result == 0 {
        //     return Err(C3p0Error::OptimisticLockError{ message: format!("Cannot update data in table [{}] with id [{}], version [{}]: data was changed!",
        //                                                                 &self.queries.qualified_table_name, &updated_model.id, &obj.version
        //     )});
        // }
        //
        // Ok(updated_model)
    }
}


#[inline]
pub fn to_model<
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send,
    CODEC: JsonCodec<DATA>,
>(
    codec: &CODEC,
    row: &DbRow,
    id_index: usize,
    version_index: usize,
    data_index: usize,
) -> Result<Model<DATA>, C3p0Error> {
    let id = row
        .try_get(id_index)
        .map_err(|err| C3p0Error::RowMapperError {
            cause: format!("Row contains no values for id index. Err: {}", err),
        })?;
    let version = row
        .try_get(version_index)
        .map_err(|err| C3p0Error::RowMapperError {
            cause: format!("Row contains no values for version index. Err: {}", err),
        })?;
    let data =
        codec.from_value(
            row.try_get(data_index)
                .map_err(|err| C3p0Error::RowMapperError {
                    cause: format!("Row contains no values for data index. Err: {}", err),
                })?,
        )?;
    Ok(Model { id, version, data })
}