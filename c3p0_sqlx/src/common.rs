/*
pub fn to_value_mapper<T: FromSqlOwned>(row: &Row) -> Result<T, Box<dyn std::error::Error>> {
    Ok(row.try_get(0).map_err(|_| C3p0Error::ResultNotFoundError)?)
}
*/


use c3p0_common::{JsonCodec, Model, C3p0Error};
use sqlx::{Row, ColumnIndex, Database};

#[inline]
pub fn to_model<
    'a,
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send,
    CODEC: JsonCodec<DATA>,
    R: Row<Database = DB>,
    IdIdx: ColumnIndex<R>,
    VersionIdx: ColumnIndex<R>,
    DataIdx: ColumnIndex<R>,
    DB: Database,
>(
    codec: &CODEC,
    row: &'a R,
    id_index: IdIdx,
    version_index: VersionIdx,
    data_index: DataIdx,
) -> Result<Model<DATA>, Box<dyn std::error::Error>>
where
    i32: sqlx::types::Type<DB> + sqlx::decode::Decode<'a, DB>,
    i64: sqlx::types::Type<DB> + sqlx::decode::Decode<'a, DB>,
    serde_json::value::Value: sqlx::types::Type<DB> + sqlx::decode::Decode<'a, DB>,
    //<DB as HasArguments<'_>>::Arguments
{
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

