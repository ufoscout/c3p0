use c3p0_common::{C3p0Error, DataType, IdType, JsonCodec, Model, VersionType};
use core::fmt::Display;
use std::borrow::Cow;
use tokio_postgres::Row;
use tokio_postgres::row::RowIndex;
use tokio_postgres::types::{FromSql, FromSqlOwned};

use crate::{IdGenerator, PostgresIdType, PostgresVersionType};

pub fn to_value_mapper<T: FromSqlOwned>(row: &Row) -> Result<T, Box<dyn std::error::Error>> {
    Ok(row.try_get(0).map_err(|_| C3p0Error::ResultNotFoundError)?)
}

/// Converts a Row into a Model using the given index positions.
///
/// - `codec`: The codec to use for serializing and deserializing the data.
/// - `id_generator`: The id generator to use for converting the DbId to an Id.
/// - `row`: The Row to convert to a Model.
/// - `id_index`: The index of the id in the row.
/// - `version_index`: The index of the version in the row.
/// - `create_epoch_millis_index`: The index of the create epoch millis in the row.
/// - `update_epoch_millis_index`: The index of the update epoch millis in the row.
/// - `data_index`: The index of the data in the row.
///
/// Returns a Model with the converted id, version, create and update epoch millis,
/// and data.
///
/// Errors if any of the positions are out of bounds, or if the id generator errors.
///
#[allow(clippy::too_many_arguments)]
#[inline]
pub fn to_model<
    Id: IdType,
    DbId: PostgresIdType,
    Data: DataType,
    CODEC: JsonCodec<Data>,
    IdIdx: RowIndex + Display,
    VersionIdx: RowIndex + Display,
    CreateEpochMillisIdx: RowIndex + Display,
    UpdateEpochMillisIdx: RowIndex + Display,
    DataIdx: RowIndex + Display,
>(
    codec: &CODEC,
    id_generator: &(dyn IdGenerator<Id, DbId>),
    row: &Row,
    id_index: IdIdx,
    version_index: VersionIdx,
    create_epoch_millis_index: CreateEpochMillisIdx,
    update_epoch_millis_index: UpdateEpochMillisIdx,
    data_index: DataIdx,
) -> Result<Model<Id, Data>, Box<dyn std::error::Error>> {
    let id: DbId = get_or_error(row, id_index)?;
    let id = id_generator.db_id_to_id(Cow::Owned(id))?.into_owned();
    let version: PostgresVersionType = get_or_error(row, version_index)?;
    let version = version as VersionType;
    let create_epoch_millis = get_or_error(row, create_epoch_millis_index)?;
    let update_epoch_millis = get_or_error(row, update_epoch_millis_index)?;
    let data = codec.data_from_value(get_or_error(row, data_index)?)?;
    Ok(Model {
        id,
        version,
        data,
        create_epoch_millis,
        update_epoch_millis,
    })
}

/// Attempts to retrieve a value of type `T` from the given `row` at the specified `index`.
///
/// - `row`: The database row from which to retrieve the value.
/// - `index`: The index in the row at which the value is expected to be found.
///
/// Returns the value of type `T` if successful, or a `C3p0Error::RowMapperError` if the value
/// cannot be retrieved, including details about the index and the encountered error.
#[inline]
pub fn get_or_error<'a, I: RowIndex + Display, T: FromSql<'a>>(
    row: &'a Row,
    index: I,
) -> Result<T, C3p0Error> {
    row.try_get(&index)
        .map_err(|err| C3p0Error::RowMapperError {
            cause: format!("Row contains no values for index {}. Err: {:?}", index, err),
        })
}
