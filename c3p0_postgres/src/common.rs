use c3p0_common::{C3p0Error, DataType, JsonCodec, Model};
use core::fmt::Display;
use tokio_postgres::row::RowIndex;
use tokio_postgres::types::{FromSql, FromSqlOwned};
use tokio_postgres::Row;

use crate::PostgresIdType;

pub fn to_value_mapper<T: FromSqlOwned>(row: &Row) -> Result<T, Box<dyn std::error::Error>> {
    Ok(row.try_get(0).map_err(|_| C3p0Error::ResultNotFoundError)?)
}

#[inline]
pub fn to_model<
    Id: PostgresIdType,
    Data: DataType,
    CODEC: JsonCodec<Data>,
    IdIdx: RowIndex + Display,
    VersionIdx: RowIndex + Display,
    CreateEpochMillisIdx: RowIndex + Display,
    UpdateEpochMillisIdx: RowIndex + Display,
    DataIdx: RowIndex + Display,
>(
    codec: &CODEC,
    row: &Row,
    id_index: IdIdx,
    version_index: VersionIdx,
    create_epoch_millis_index: CreateEpochMillisIdx,
    update_epoch_millis_index: UpdateEpochMillisIdx,
    data_index: DataIdx,
) -> Result<Model<Id, Data>, Box<dyn std::error::Error>> {
    let id = get_or_error(row, id_index)?;
    let version = get_or_error(row, version_index)?;
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
