use c3p0_common::{C3p0Error, DataType, IdType, JsonCodec, Model, VersionType};
use sqlx::{ColumnIndex, Database, Decode, Row, Type, query::Query};

pub type SqlxVersionType = i32;

/// A trait that allows the creation of an Id
pub trait IdGenerator<Id: IdType>: Send + Sync + 'static {
    type Db: Database;
    type Row: Row<Database = Self::Db>;

    /// Returns the type of the column that will be used to store the Id
    fn create_statement_column_type(&self) -> &str;

    /// Returns the generated Id
    fn generate_id(&self) -> Option<Id>;

    /// Binds the Id to the query
    fn id_to_query<'a>(
        &self,
        id: &'a Id,
        query: Query<'a, Self::Db, <Self::Db as Database>::Arguments<'a>>,
    ) -> Query<'a, Self::Db, <Self::Db as Database>::Arguments<'a>>;

    /// Extracts the Id from the row
    fn id_from_row(
        &self,
        row: &Self::Row,
        index: &(dyn sqlx::ColumnIndex<Self::Row>),
    ) -> Result<Id, C3p0Error>;
}

/// Converts a row to a Model
#[inline]
pub fn to_model<
    Id: IdType,
    Data: DataType,
    CODEC: JsonCodec<Data>,
    R: Row<Database = DB>,
    DB: Database,
>(
    codec: &CODEC,
    id_generator: &(dyn IdGenerator<Id, Db = DB, Row = R>),
    row: &R,
) -> Result<Model<Id, Data>, C3p0Error>
where
    usize: ColumnIndex<R>,
    for<'c> i32: Type<DB> + Decode<'c, DB>,
    for<'c> i64: Type<DB> + Decode<'c, DB>,
    for<'c> serde_json::value::Value: Type<DB> + Decode<'c, DB>,
{
    to_model_with_index(codec, id_generator, row, 0, 1, 2, 3, 4)
}

/// Converts a row to a Model
#[allow(clippy::too_many_arguments)]
#[inline]
pub fn to_model_with_index<
    Id: IdType,
    Data: DataType,
    CODEC: JsonCodec<Data>,
    R: Row<Database = DB>,
    IdIdx: ColumnIndex<R>,
    VersionIdx: ColumnIndex<R>,
    CreateEpochMillisIdx: ColumnIndex<R>,
    UpdateEpochMillisIdx: ColumnIndex<R>,
    DataIdx: ColumnIndex<R>,
    DB: Database,
>(
    codec: &CODEC,
    id_generator: &(dyn IdGenerator<Id, Db = DB, Row = R>),
    row: &R,
    id_index: IdIdx,
    version_index: VersionIdx,
    create_epoch_millis_index: CreateEpochMillisIdx,
    update_epoch_millis_index: UpdateEpochMillisIdx,
    data_index: DataIdx,
) -> Result<Model<Id, Data>, C3p0Error>
where
    for<'c> i32: Type<DB> + Decode<'c, DB>,
    for<'c> i64: Type<DB> + Decode<'c, DB>,
    for<'c> serde_json::value::Value: Type<DB> + Decode<'c, DB>,
{
    let id = id_generator.id_from_row(row, &id_index)?;

    let version: SqlxVersionType =
        row.try_get(version_index)
            .map_err(|err| C3p0Error::RowMapperError {
                cause: format!("Row contains no values for version index. Err: {:?}", err),
            })?;
    let version = version as VersionType;
    let create_epoch_millis =
        row.try_get(create_epoch_millis_index)
            .map_err(|err| C3p0Error::RowMapperError {
                cause: format!(
                    "Row contains no values for create_epoch_millis index. Err: {:?}",
                    err
                ),
            })?;
    let update_epoch_millis =
        row.try_get(update_epoch_millis_index)
            .map_err(|err| C3p0Error::RowMapperError {
                cause: format!(
                    "Row contains no values for update_epoch_millis index. Err: {:?}",
                    err
                ),
            })?;
    let data = codec.data_from_value(row.try_get(data_index).map_err(|err| {
        C3p0Error::RowMapperError {
            cause: format!("Row contains no values for data index. Err: {:?}", err),
        }
    })?)?;
    Ok(Model {
        id,
        version,
        data,
        create_epoch_millis,
        update_epoch_millis,
    })
}
