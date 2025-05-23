use std::borrow::Cow;
use std::sync::Arc;

use crate::tokio_postgres::{row::Row, types::ToSql};
use crate::*;
use ::tokio_postgres::types::FromSqlOwned;
use c3p0_common::json::Queries;
use c3p0_common::time::utils::get_current_epoch_millis;
use c3p0_common::*;

pub trait PostgresIdType: IdType + FromSqlOwned + ToSql {}
impl<T: IdType + FromSqlOwned + ToSql> PostgresIdType for T {}

pub type PostgresVersionType = i32;

/// A trait that allows the creation of an Id
pub trait IdGenerator<Id: IdType, DbId: PostgresIdType>: Send + Sync {
    /// Returns the column type for the id in the create statement
    fn create_statement_column_type(&self) -> &str;
    /// Generates a new id
    fn generate_id(&self) -> Option<DbId>;
    /// Converts an Id to a DbId
    fn id_to_db_id<'a>(&self, id: Cow<'a, Id>) -> Result<Cow<'a, DbId>, C3p0Error>;
    /// Converts a DbId to an Id
    fn db_id_to_id<'a>(&self, id: Cow<'a, DbId>) -> Result<Cow<'a, Id>, C3p0Error>;
}

/// An IdGenerator that uses the auto-increment feature of the database
pub struct AutogeneratedIdGenerator {}

impl IdGenerator<u64, i64> for AutogeneratedIdGenerator {
    fn create_statement_column_type(&self) -> &str {
        "bigserial"
    }

    fn generate_id(&self) -> Option<i64> {
        None
    }

    fn id_to_db_id<'a>(&self, id: Cow<'a, u64>) -> Result<Cow<'a, i64>, C3p0Error> {
        Ok(Cow::Owned(id.into_owned() as i64))
    }

    fn db_id_to_id<'a>(&self, id: Cow<'a, i64>) -> Result<Cow<'a, u64>, C3p0Error> {
        Ok(Cow::Owned(id.into_owned() as u64))
    }
}

/// An IdGenerator that uses the uuid crate to generate a random uuid
pub struct UuidIdGenerator {}

impl IdGenerator<uuid::Uuid, uuid::Uuid> for UuidIdGenerator {
    fn create_statement_column_type(&self) -> &str {
        "uuid"
    }

    fn generate_id(&self) -> Option<uuid::Uuid> {
        Some(uuid::Uuid::new_v4())
    }

    fn id_to_db_id<'a>(&self, id: Cow<'a, uuid::Uuid>) -> Result<Cow<'a, uuid::Uuid>, C3p0Error> {
        Ok(id)
    }

    fn db_id_to_id<'a>(&self, id: Cow<'a, uuid::Uuid>) -> Result<Cow<'a, uuid::Uuid>, C3p0Error> {
        Ok(id)
    }
}

/// A builder for a PgC3p0Json
#[derive(Clone)]
pub struct PgC3p0JsonBuilder<Id: IdType, DbId: PostgresIdType> {
    pub id_generator: Arc<dyn IdGenerator<Id, DbId>>,
    pub id_field_name: String,
    pub version_field_name: String,
    pub create_epoch_millis_field_name: String,
    pub update_epoch_millis_field_name: String,
    pub data_field_name: String,
    pub table_name: String,
    pub schema_name: Option<String>,
}

impl PgC3p0JsonBuilder<u64, i64> {
    /// Creates a new PgC3p0JsonBuilder for a table with the given name
    pub fn new<T: Into<String>>(table_name: T) -> Self {
        let table_name = table_name.into();
        PgC3p0JsonBuilder {
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

impl<Id: IdType, DbId: PostgresIdType> PgC3p0JsonBuilder<Id, DbId> {
    /// Sets the id field name
    pub fn with_id_field_name<T: Into<String>>(mut self, id_field_name: T) -> Self {
        self.id_field_name = id_field_name.into();
        self
    }

    /// Sets the version field name
    pub fn with_version_field_name<T: Into<String>>(mut self, version_field_name: T) -> Self {
        self.version_field_name = version_field_name.into();
        self
    }

    /// Sets the create_epoch_millis field name
    pub fn with_create_epoch_millis_field_name<T: Into<String>>(
        mut self,
        create_epoch_millis_field_name: T,
    ) -> Self {
        self.create_epoch_millis_field_name = create_epoch_millis_field_name.into();
        self
    }

    /// Sets the update_epoch_millis field name
    pub fn with_update_epoch_millis_field_name<T: Into<String>>(
        mut self,
        update_epoch_millis_field_name: T,
    ) -> Self {
        self.update_epoch_millis_field_name = update_epoch_millis_field_name.into();
        self
    }

    /// Sets the data field name
    pub fn with_data_field_name<T: Into<String>>(mut self, data_field_name: T) -> Self {
        self.data_field_name = data_field_name.into();
        self
    }

    /// Sets the schema name
    pub fn with_schema_name<O: Into<Option<String>>>(mut self, schema_name: O) -> Self {
        self.schema_name = schema_name.into();
        self
    }

    /// Sets the id generator
    pub fn with_id_generator<
        NewId: IdType,
        NewDbId: PostgresIdType,
        T: 'static + IdGenerator<NewId, NewDbId> + Send + Sync,
    >(
        self,
        id_generator: T,
    ) -> PgC3p0JsonBuilder<NewId, NewDbId> {
        PgC3p0JsonBuilder {
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

    /// Builds a PgC3p0Json
    pub fn build<Data: DataType>(self) -> PgC3p0Json<Id, DbId, Data, DefaultJsonCodec> {
        self.build_with_codec(DefaultJsonCodec {})
    }

    /// Builds a PgC3p0Json with the given codec
    pub fn build_with_codec<Data: DataType, CODEC: JsonCodec<Data>>(
        self,
        codec: CODEC,
    ) -> PgC3p0Json<Id, DbId, Data, CODEC> {
        PgC3p0Json {
            phantom_data: std::marker::PhantomData,
            id_generator: self.id_generator.clone(),
            codec,
            queries: build_pg_queries(self),
        }
    }
}

/// A C3p0Json implementation for Postgres
#[derive(Clone)]
pub struct PgC3p0Json<Id: IdType, DbId: PostgresIdType, Data: DataType, CODEC: JsonCodec<Data>> {
    phantom_data: std::marker::PhantomData<Data>,
    id_generator: Arc<dyn IdGenerator<Id, DbId>>,
    codec: CODEC,
    queries: Queries,
}

impl<Id: IdType, DbId: PostgresIdType, Data: DataType, CODEC: JsonCodec<Data>>
    PgC3p0Json<Id, DbId, Data, CODEC>
{
    /// Returns the Postgres specific queries for this C3p0Json
    pub fn queries(&self) -> &Queries {
        &self.queries
    }

    /// Converts a Postgres row to a Model
    #[inline]
    pub fn to_model(&self, row: &Row) -> Result<Model<Id, Data>, Box<dyn std::error::Error>> {
        to_model(&self.codec, self.id_generator.as_ref(), row, 0, 1, 2, 3, 4)
    }

    /// Allows the execution of a custom sql query and returns the first entry in the result set.
    /// For this to work, the sql query:
    /// - must be a SELECT
    /// - must declare the ID, VERSION and Data fields in this exact order
    pub async fn fetch_one_optional_with_sql(
        &self,
        tx: &mut PgTx<'_>,
        sql: &str,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<Option<Model<Id, Data>>, C3p0Error> {
        tx.fetch_one_optional(sql, params, |row| self.to_model(row))
            .await
    }

    /// Allows the execution of a custom sql query and returns the first entry in the result set.
    /// For this to work, the sql query:
    /// - must be a SELECT
    /// - must declare the ID, VERSION and Data fields in this exact order
    pub async fn fetch_one_with_sql(
        &self,
        tx: &mut PgTx<'_>,
        sql: &str,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<Model<Id, Data>, C3p0Error> {
        tx.fetch_one(sql, params, |row| self.to_model(row)).await
    }

    /// Allows the execution of a custom sql query and returns all the entries in the result set.
    /// For this to work, the sql query:
    /// - must be a SELECT
    /// - must declare the ID, VERSION and Data fields in this exact order
    pub async fn fetch_all_with_sql(
        &self,
        tx: &mut PgTx<'_>,
        sql: &str,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<Vec<Model<Id, Data>>, C3p0Error> {
        tx.fetch_all(sql, params, |row| self.to_model(row)).await
    }
}

impl<Id: IdType, DbId: PostgresIdType, Data: DataType, CODEC: JsonCodec<Data>>
    C3p0Json<Id, Data, CODEC> for PgC3p0Json<Id, DbId, Data, CODEC>
{
    type Tx<'a> = PgTx<'a>;

    fn codec(&self) -> &CODEC {
        &self.codec
    }

    async fn create_table_if_not_exists(&self, tx: &mut Self::Tx<'_>) -> Result<(), C3p0Error> {
        tx.execute(&self.queries.create_table_sql_query, &[])
            .await?;
        Ok(())
    }

    async fn drop_table_if_exists(
        &self,
        tx: &mut Self::Tx<'_>,
        cascade: bool,
    ) -> Result<(), C3p0Error> {
        let query = if cascade {
            &self.queries.drop_table_sql_query_cascade
        } else {
            &self.queries.drop_table_sql_query
        };
        tx.execute(query, &[]).await?;
        Ok(())
    }

    async fn count_all(&self, tx: &mut Self::Tx<'_>) -> Result<u64, C3p0Error> {
        tx.fetch_one_value(&self.queries.count_all_sql_query, &[])
            .await
            .map(|val: i64| val as u64)
    }

    async fn exists_by_id(&self, tx: &mut Self::Tx<'_>, id: &Id) -> Result<bool, C3p0Error> {
        let id = self.id_generator.id_to_db_id(Cow::Borrowed(id))?;
        tx.fetch_one_value(&self.queries.exists_by_id_sql_query, &[id.as_ref()])
            .await
    }

    async fn fetch_all(&self, tx: &mut Self::Tx<'_>) -> Result<Vec<Model<Id, Data>>, C3p0Error> {
        tx.fetch_all(&self.queries.find_all_sql_query, &[], |row| {
            self.to_model(row)
        })
        .await
    }

    async fn fetch_one_optional_by_id(
        &self,
        tx: &mut Self::Tx<'_>,
        id: &Id,
    ) -> Result<Option<Model<Id, Data>>, C3p0Error> {
        let id = self.id_generator.id_to_db_id(Cow::Borrowed(id))?;
        tx.fetch_one_optional(&self.queries.find_by_id_sql_query, &[id.as_ref()], |row| {
            self.to_model(row)
        })
        .await
    }

    async fn fetch_one_by_id(
        &self,
        tx: &mut Self::Tx<'_>,
        id: &Id,
    ) -> Result<Model<Id, Data>, C3p0Error> {
        self.fetch_one_optional_by_id(tx, id)
            .await
            .and_then(|result| result.ok_or(C3p0Error::ResultNotFoundError))
    }

    async fn delete(
        &self,
        tx: &mut Self::Tx<'_>,
        obj: Model<Id, Data>,
    ) -> Result<Model<Id, Data>, C3p0Error> {
        let id = self.id_generator.id_to_db_id(Cow::Borrowed(&obj.id))?;
        let result = tx
            .execute(
                &self.queries.delete_sql_query,
                &[id.as_ref(), &(obj.version as PostgresVersionType)],
            )
            .await?;

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

    async fn delete_all(&self, tx: &mut Self::Tx<'_>) -> Result<u64, C3p0Error> {
        tx.execute(&self.queries.delete_all_sql_query, &[]).await
    }

    async fn delete_by_id(&self, tx: &mut Self::Tx<'_>, id: &Id) -> Result<u64, C3p0Error> {
        let id = self.id_generator.id_to_db_id(Cow::Borrowed(id))?;
        tx.execute(&self.queries.delete_by_id_sql_query, &[id.as_ref()])
            .await
    }

    async fn save(
        &self,
        tx: &mut Self::Tx<'_>,
        obj: NewModel<Data>,
    ) -> Result<Model<Id, Data>, C3p0Error> {
        let json_data = &self.codec.data_to_value(&obj.data)?;
        let create_epoch_millis = get_current_epoch_millis();

        let id = match self.id_generator.generate_id() {
            Some(id) => {
                tx.execute(
                    &self.queries.save_sql_query_with_id,
                    &[
                        &(obj.version as PostgresVersionType),
                        &create_epoch_millis,
                        &json_data,
                        &id,
                    ],
                )
                .await?;
                id
            }
            _ => {
                tx.fetch_one_value(
                    &self.queries.save_sql_query,
                    &[
                        &(obj.version as PostgresVersionType),
                        &create_epoch_millis,
                        &json_data,
                    ],
                )
                .await?
            }
        };

        Ok(Model {
            id: self.id_generator.db_id_to_id(Cow::Owned(id))?.into_owned(),
            version: obj.version,
            data: obj.data,
            create_epoch_millis,
            update_epoch_millis: create_epoch_millis,
        })
    }

    async fn update(
        &self,
        tx: &mut Self::Tx<'_>,
        obj: Model<Id, Data>,
    ) -> Result<Model<Id, Data>, C3p0Error> {
        let json_data = &self.codec.data_to_value(&obj.data)?;
        let previous_version = obj.version;
        let updated_model = obj.into_new_version(get_current_epoch_millis());
        let updated_model_id = self
            .id_generator
            .id_to_db_id(Cow::Borrowed(&updated_model.id))?;
        let result = tx
            .execute(
                &self.queries.update_sql_query,
                &[
                    &(updated_model.version as PostgresVersionType),
                    &updated_model.update_epoch_millis,
                    &json_data,
                    updated_model_id.as_ref(),
                    &(previous_version as PostgresVersionType),
                ],
            )
            .await?;

        if result == 0 {
            return Err(C3p0Error::OptimisticLockError {
                cause: format!(
                    "Cannot update data in table [{}] with id [{:?}], version [{}]: data was changed!",
                    &self.queries.qualified_table_name, &updated_model.id, &previous_version
                ),
            });
        }

        Ok(updated_model)
    }
}
