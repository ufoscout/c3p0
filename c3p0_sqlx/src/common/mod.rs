pub mod executor;

use c3p0_common::{C3p0Error, JsonCodec, Model};
use sqlx::{ColumnIndex, Database, Row};

#[inline]
pub fn to_model<
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send,
    CODEC: JsonCodec<DATA>,
    R: Row<Database = DB>,
    IdIdx: ColumnIndex<R>,
    VersionIdx: ColumnIndex<R>,
    DataIdx: ColumnIndex<R>,
    DB: Database,
>(
    codec: &CODEC,
    row: &R,
    id_index: IdIdx,
    version_index: VersionIdx,
    data_index: DataIdx,
) -> Result<Model<DATA>, C3p0Error>
where
    for<'c> i32: sqlx::types::Type<DB> + sqlx::decode::Decode<'c, DB>,
    for<'c> i64: sqlx::types::Type<DB> + sqlx::decode::Decode<'c, DB>,
    for<'c> serde_json::value::Value: sqlx::types::Type<DB> + sqlx::decode::Decode<'c, DB>,
    //<DB as HasArguments<'_>>::Arguments
{
    let id = row
        .try_get(id_index)
        .map_err(|err| C3p0Error::RowMapperError {
            cause: format!("Row contains no values for id index. Err: {:?}", err),
        })?;
    let version = row
        .try_get(version_index)
        .map_err(|err| C3p0Error::RowMapperError {
            cause: format!("Row contains no values for version index. Err: {:?}", err),
        })?;
    let data =
        codec.from_value(
            row.try_get(data_index)
                .map_err(|err| C3p0Error::RowMapperError {
                    cause: format!("Row contains no values for data index. Err: {:?}", err),
                })?,
        )?;
    Ok(Model { id, version, data })
}
