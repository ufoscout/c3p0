use core::panic;
use std::fmt::Debug;
use std::sync::Arc;

use crate::common::to_model;
use crate::error::into_c3p0_error;
use crate::mysql::queries::build_mysql_queries;
use crate::mysql::{Db, DbRow, MySqlTx};
use async_trait::async_trait;
use c3p0_common::json::Queries;
use c3p0_common::time::utils::get_current_epoch_millis;
use c3p0_common::*;
use sqlx::query::Query;
use sqlx::{IntoArguments, Row, Encode, Decode, Type};

/// A trait that allows the creation of an Id
pub trait IdGenerator<Id>: Send + Sync {
    fn generate_id(&self) -> Option<Id>;
}

/// An IdGenerator that uses the auto-increment feature of the database
pub struct AutogeneratedIdGenerator {}

impl IdGenerator<i64> for AutogeneratedIdGenerator {
    fn generate_id(&self) -> Option<i64> {
        None
    }
}

/// An IdGenerator that uses the uuid crate to generate a random uuid
pub struct UuidIdGenerator {}

impl IdGenerator<uuid::Uuid> for UuidIdGenerator {
    fn generate_id(&self) -> Option<uuid::Uuid> {
        Some(uuid::Uuid::new_v4())
    }
}

#[derive(Clone)]
pub struct SqlxMySqlC3p0JsonBuilder<Id> {
    pub id_generator: Arc<dyn IdGenerator<Id>>,
    pub id_field_name: String,
    pub version_field_name: String,
    pub create_epoch_millis_field_name: String,
    pub update_epoch_millis_field_name: String,
    pub data_field_name: String,
    pub table_name: String,
    pub schema_name: Option<String>,
}

impl SqlxMySqlC3p0JsonBuilder<i64> {
    pub fn new<T: Into<String>>(table_name: T) -> Self {
        let table_name = table_name.into();
        SqlxMySqlC3p0JsonBuilder {
            id_generator: Arc::new(AutogeneratedIdGenerator {}),
            table_name,
            id_field_name: "id".to_owned(),
            version_field_name: "version".to_owned(),
            create_epoch_millis_field_name: "create_epoch_millis".to_owned(),
            update_epoch_millis_field_name: "update_epoch_millis".to_owned(),
            data_field_name: "data".to_owned(),
            schema_name: None,
        }
    }
}

impl <Id: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send + Sync + Type<Db> + Debug> SqlxMySqlC3p0JsonBuilder<Id>
where
for<'c> Id: Encode<'c, Db> + Decode<'c, Db>
{

    pub fn with_id_field_name<T: Into<String>>(mut self, id_field_name: T) -> Self {
        self.id_field_name = id_field_name.into();
        self
    }

    pub fn with_version_field_name<T: Into<String>>(mut self, version_field_name: T) -> Self {
        self.version_field_name = version_field_name.into();
        self
    }

    pub fn with_create_epoch_millis_field_name<T: Into<String>>(
        mut self,
        create_epoch_millis_field_name: T,
    ) -> Self {
        self.create_epoch_millis_field_name = create_epoch_millis_field_name.into();
        self
    }

    pub fn with_update_epoch_millis_field_name<T: Into<String>>(
        mut self,
        update_epoch_millis_field_name: T,
    ) -> Self {
        self.update_epoch_millis_field_name = update_epoch_millis_field_name.into();
        self
    }

    pub fn with_data_field_name<T: Into<String>>(mut self, data_field_name: T) -> Self {
        self.data_field_name = data_field_name.into();
        self
    }

    pub fn with_schema_name<O: Into<Option<String>>>(mut self, schema_name: O) -> Self {
        self.schema_name = schema_name.into();
        self
    }

    pub fn with_id_generator<NewId, T: 'static + IdGenerator<NewId> + Send + Sync>(
        self,
        id_generator: T,
    ) -> SqlxMySqlC3p0JsonBuilder<NewId> {
        SqlxMySqlC3p0JsonBuilder {
            id_generator: Arc::new(id_generator),
            id_field_name: self.id_field_name,
            version_field_name: self.version_field_name,
            create_epoch_millis_field_name: self.create_epoch_millis_field_name,
            update_epoch_millis_field_name: self.update_epoch_millis_field_name,
            data_field_name: self.data_field_name,
            table_name: self.table_name,
            schema_name: self.schema_name,
        }
    }

    pub fn build<
        Data: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send + Sync>(
        self,
    ) -> SqlxMySqlC3p0Json<Id, Data, DefaultJsonCodec> {
        self.build_with_codec(DefaultJsonCodec {})
    }

    pub fn build_with_codec<
        Data: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send + Sync,
        CODEC: JsonCodec<Data>,
    >(
        self,
        codec: CODEC,
    ) -> SqlxMySqlC3p0Json<Id, Data, CODEC> {
        SqlxMySqlC3p0Json {
            phantom_data: std::marker::PhantomData,
            id_generator: self.id_generator.clone(),
            codec,
            queries: build_mysql_queries(self),
        }
    }
}

#[derive(Clone)]
pub struct SqlxMySqlC3p0Json<Id, Data, CODEC: JsonCodec<Data>>
where
    Id: 'static + Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send + Sync + Type<Db> + Debug,
    Data: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send + Sync,
for<'c> Id: Encode<'c, Db> + Decode<'c, Db>
{
    phantom_data: std::marker::PhantomData<Data>,
    id_generator: Arc<dyn IdGenerator<Id>>,
    codec: CODEC,
    queries: Queries,
}

impl<Id, Data, CODEC: JsonCodec<Data>> SqlxMySqlC3p0Json<Id, Data, CODEC>
where
    Id: 'static + Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send + Sync + Type<Db> + Debug,
    Data: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send + Sync,
for<'c> Id: Encode<'c, Db> + Decode<'c, Db>
{
    pub fn queries(&self) -> &Queries {
        &self.queries
    }

    #[inline]
    pub fn to_model(&self, row: &DbRow) -> Result<Model<Id, Data>, C3p0Error> {
        to_model(&self.codec, row, 0, 1, 2, 3, 4)
    }

    /// Allows the execution of a custom sql query and returns the first entry in the result set.
    /// For this to work, the sql query:
    /// - must be a SELECT
    /// - must declare the ID, VERSION and Data fields in this exact order
    pub async fn fetch_one_optional_with_sql<'a, A: 'a + Send + IntoArguments<'a, Db>>(
        &self,
        tx: &mut MySqlTx,
        sql: Query<'a, Db, A>,
    ) -> Result<Option<Model<Id, Data>>, C3p0Error> {
        sql
        .fetch_optional(tx.conn())
        .await
        .map_err(into_c3p0_error)?
        .map(|row| to_model(&self.codec, &row, 0, 1, 2, 3, 4))
        .transpose()
    }

    /// Allows the execution of a custom sql query and returns the first entry in the result set.
    /// For this to work, the sql query:
    /// - must be a SELECT
    /// - must declare the ID, VERSION and Data fields in this exact order
    pub async fn fetch_one_with_sql<'a, A: 'a + Send + IntoArguments<'a, Db>>(
        &self,
        tx: &mut MySqlTx,
        sql: Query<'a, Db, A>,
    ) -> Result<Model<Id, Data>, C3p0Error> {
        sql
        .fetch_one(tx.conn())
        .await
        .map_err(into_c3p0_error)
        .and_then(|row| to_model(self.codec(), &row, 0, 1, 2, 3, 4))
    }

    /// Allows the execution of a custom sql query and returns all the entries in the result set.
    /// For this to work, the sql query:
    /// - must be a SELECT
    /// - must declare the ID, VERSION and Data fields in this exact order
    pub async fn fetch_all_with_sql<'a, A: 'a + Send + IntoArguments<'a, Db>>(
        &self,
        tx: &mut MySqlTx,
        sql: Query<'a, Db, A>,
    ) -> Result<Vec<Model<Id, Data>>, C3p0Error> {
        sql
        .fetch_all(tx.conn())
        .await
        .map_err(into_c3p0_error)?
        .iter()
        .map(|row| to_model(self.codec(), row, 0, 1, 2, 3, 4))
        .collect::<Result<Vec<_>, C3p0Error>>()
    }
}

#[async_trait]
impl<Id, Data, CODEC: JsonCodec<Data>> C3p0Json<Id, Data, CODEC> for SqlxMySqlC3p0Json<Id, Data, CODEC>
where
    Id: 'static + Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send + Sync + Type<Db> + Debug,
    Data: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send + Sync,
    for<'c> Id: Encode<'c, Db> + Decode<'c, Db>,
{
    type Tx = MySqlTx;

    fn codec(&self) -> &CODEC {
        &self.codec
    }

    async fn create_table_if_not_exists(&self, tx: &mut Self::Tx) -> Result<(), C3p0Error> {
        sqlx::query(&self.queries.create_table_sql_query)
        .execute(tx.conn())
        .await
        .map_err(into_c3p0_error)
        .map(|_| ())
    }

    async fn drop_table_if_exists(
        &self,
        tx: &mut Self::Tx,
        cascade: bool,
    ) -> Result<(), C3p0Error> {
        let query = if cascade {
            &self.queries.drop_table_sql_query_cascade
        } else {
            &self.queries.drop_table_sql_query
        };
        sqlx::query(&query)
        .execute(tx.conn())
        .await
        .map_err(into_c3p0_error)
        .map(|_| ())
    }

    async fn count_all(&self, tx: &mut Self::Tx) -> Result<u64, C3p0Error> {
        sqlx::query(&self.queries.count_all_sql_query)
            .fetch_one(tx.conn())
            .await
            .and_then(|row| row.try_get(0))
            .map_err(into_c3p0_error)
            .map(|val: i64| val as u64)
    }

    async fn exists_by_id<'a, ID: Into<&'a Id> + Send>(
        &'a self,
        tx: &mut Self::Tx,
        id: ID,
    ) -> Result<bool, C3p0Error> {
        sqlx::query(&self.queries.exists_by_id_sql_query)
            .bind(id.into())
            .fetch_one(tx.conn())
            .await
            .and_then(|row| row.try_get(0))
            .map_err(into_c3p0_error)
    }

    async fn fetch_all(&self, tx: &mut Self::Tx) -> Result<Vec<Model<Id, Data>>, C3p0Error> {
        self.fetch_all_with_sql(tx, sqlx::query(&self.queries.find_all_sql_query))
            .await
    }

    async fn fetch_one_optional_by_id<'a, ID: Into<&'a Id> + Send>(
        &'a self,
        tx: &mut Self::Tx,
        id: ID,
    ) -> Result<Option<Model<Id, Data>>, C3p0Error> {
        self.fetch_one_optional_with_sql(
            tx,
            sqlx::query(&self.queries.find_by_id_sql_query).bind(id.into()),
        )
        .await
    }

    async fn fetch_one_by_id<'a, ID: Into<&'a Id> + Send>(
        &'a self,
        tx: &mut Self::Tx,
        id: ID,
    ) -> Result<Model<Id, Data>, C3p0Error> {
        self.fetch_one_with_sql(
            tx,
            sqlx::query(&self.queries.find_by_id_sql_query).bind(id.into()),
        )
        .await
    }

    async fn delete(&self, tx: &mut Self::Tx, obj: Model<Id, Data>) -> Result<Model<Id, Data>, C3p0Error> {
        let result = sqlx::query(&self.queries.delete_sql_query)
        .bind(obj.id.clone())
        .bind(obj.version)
        .execute(tx.conn())
        .await
        .map_err(into_c3p0_error)?
        .rows_affected();

    if result == 0 {
        return Err(C3p0Error::OptimisticLockError {
            cause: format!(
                "Cannot delete data in table [{}] with id [{:?}], version [{}]: data was changed!",
                &self.queries.qualified_table_name, &obj.id, &obj.version
            ),
        });
    }

    Ok(obj)
    }

    async fn delete_all(&self, tx: &mut Self::Tx) -> Result<u64, C3p0Error> {
        sqlx::query(&self.queries.delete_all_sql_query)
            .execute(tx.conn())
            .await
            .map_err(into_c3p0_error)
            .map(|done| done.rows_affected())
    }

    async fn delete_by_id<'a, ID: Into<&'a Id> + Send>(
        &'a self,
        tx: &mut Self::Tx,
        id: ID,
    ) -> Result<u64, C3p0Error> {
        sqlx::query(&self.queries.delete_by_id_sql_query)
            .bind(id.into())
            .execute(tx.conn())
            .await
            .map_err(into_c3p0_error)
            .map(|done| done.rows_affected())
    }

    async fn save(&self, tx: &mut Self::Tx, obj: NewModel<Data>) -> Result<Model<Id, Data>, C3p0Error> {
        let json_data = self.codec().data_to_value(&obj.data)?;
        let create_epoch_millis = get_current_epoch_millis();

        let id = if let Some(id) = self.id_generator.generate_id() {
            sqlx::query(&self.queries.save_sql_query_with_id)
                .bind(obj.version)
                .bind(create_epoch_millis)
                .bind(create_epoch_millis)
                .bind(&json_data)
                .execute(tx.conn())
                .await
                .map_err(into_c3p0_error)?;
            id
        } else {
            let id = sqlx::query(&self.queries.save_sql_query)
                .bind(obj.version)
                .bind(create_epoch_millis)
                .bind(create_epoch_millis)
                .bind(&json_data)
                .execute(tx.conn())
                .await
                .map(|done| done.last_insert_id())
                .map_err(into_c3p0_error)?;
            downcast_to_id(Box::new(id as i64))
        };

        Ok(Model {
            id,
            version: obj.version,
            data: obj.data,
            create_epoch_millis,
            update_epoch_millis: create_epoch_millis,
        })
    }

    async fn update(&self, tx: &mut Self::Tx, obj: Model<Id, Data>) -> Result<Model<Id, Data>, C3p0Error> {
        let json_data = self.codec.data_to_value(&obj.data)?;
        let previous_version = obj.version;
        let updated_model = obj.into_new_version(get_current_epoch_millis());
    
        let result = {
            sqlx::query(&self.queries.update_sql_query)
                .bind(updated_model.version)
                .bind(updated_model.update_epoch_millis)
                .bind(json_data)
                .bind(updated_model.id.clone())
                .bind(previous_version)
                .execute(tx.conn())
                .await
                .map_err(into_c3p0_error)
                .map(|done| done.rows_affected())?
        };
    
        if result == 0 {
            return Err(C3p0Error::OptimisticLockError {
                cause: format!(
                    "Cannot update data in table [{}] with id [{:?}], version [{}]: data was changed!",
                    self.queries.qualified_table_name, updated_model.id, &previous_version
                ),
            });
        }
    
        Ok(updated_model)
    }
}

fn downcast_to_id<Id: 'static>(value: Box<dyn std::any::Any>) -> Id {
    if let Ok(value) = value.downcast::<Id>() {
        *value
    } else {
        panic!("Mysql Autogenerated Id must be of type i64")
    }
}
