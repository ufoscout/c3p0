use crate::common::to_model;
use crate::error::into_c3p0_error;
use c3p0_common::json::Queries;
use c3p0_common::time::utils::get_current_epoch_millis;
use c3p0_common::{C3p0Error, JsonCodec, Model};
use sqlx::query::Query;
use sqlx::{ColumnIndex, Database, Executor, IntoArguments};

pub trait ResultWithRowCount {
    fn rows_affected(&self) -> u64;
}

#[inline]
pub async fn batch_execute<'e, 'q: 'e, E, DB>(query: &'q str, executor: E) -> Result<(), C3p0Error>
where
    DB: Database,
    <DB as sqlx::database::HasArguments<'q>>::Arguments: sqlx::IntoArguments<'q, DB>,
    E: Executor<'e, Database = DB>,
{
    executor
        .execute(query)
        .await
        .map_err(into_c3p0_error)
        .map(|_| ())
}

#[inline]
pub async fn execute<'e, 'q: 'e, A, E, DB>(
    query: Query<'q, DB, A>,
    executor: E,
) -> Result<(), C3p0Error>
where
    DB: Database,
    A: 'q + IntoArguments<'q, DB>,
    E: Executor<'e, Database = DB>,
{
    query
        .execute(executor)
        .await
        .map_err(into_c3p0_error)
        .map(|_| ())
}

#[inline]
pub async fn fetch_one_optional_with_sql<'e, 'q: 'e, A, E, DB, DATA, CODEC: JsonCodec<DATA>>(
    query: Query<'q, DB, A>,
    executor: E,
    codec: &CODEC,
) -> Result<Option<Model<DATA>>, C3p0Error>
where
    DB: Database,
    A: 'q + IntoArguments<'q, DB>,
    E: Executor<'e, Database = DB>,
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send + Sync,
    for<'c> i32: sqlx::types::Type<DB> + sqlx::decode::Decode<'c, DB>,
    for<'c> i64: sqlx::types::Type<DB> + sqlx::decode::Decode<'c, DB>,
    for<'c> serde_json::value::Value: sqlx::types::Type<DB> + sqlx::decode::Decode<'c, DB>,
    usize: ColumnIndex<DB::Row>,
{
    query
        .fetch_optional(executor)
        .await
        .map_err(into_c3p0_error)?
        .map(|row| to_model(codec, &row, 0, 1, 2, 3, 4))
        .transpose()
}

#[inline]
pub async fn fetch_one_with_sql<'e, 'q: 'e, A, E, DB, DATA, CODEC: JsonCodec<DATA>>(
    query: Query<'q, DB, A>,
    executor: E,
    codec: &CODEC,
) -> Result<Model<DATA>, C3p0Error>
where
    DB: Database,
    A: 'q + IntoArguments<'q, DB>,
    E: Executor<'e, Database = DB>,
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send + Sync,
    for<'c> i32: sqlx::types::Type<DB> + sqlx::decode::Decode<'c, DB>,
    for<'c> i64: sqlx::types::Type<DB> + sqlx::decode::Decode<'c, DB>,
    for<'c> serde_json::value::Value: sqlx::types::Type<DB> + sqlx::decode::Decode<'c, DB>,
    usize: ColumnIndex<DB::Row>,
{
    query
        .fetch_one(executor)
        .await
        .map_err(into_c3p0_error)
        .and_then(|row| to_model(codec, &row, 0, 1, 2, 3, 4))
}

#[inline]
pub async fn fetch_all_with_sql<'e, 'q: 'e, A, E, DB, DATA, CODEC: JsonCodec<DATA>>(
    query: Query<'q, DB, A>,
    executor: E,
    codec: &CODEC,
) -> Result<Vec<Model<DATA>>, C3p0Error>
where
    DB: Database,
    A: 'q + IntoArguments<'q, DB>,
    E: Executor<'e, Database = DB>,
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send + Sync,
    for<'c> i32: sqlx::types::Type<DB> + sqlx::decode::Decode<'c, DB>,
    for<'c> i64: sqlx::types::Type<DB> + sqlx::decode::Decode<'c, DB>,
    for<'c> serde_json::value::Value: sqlx::types::Type<DB> + sqlx::decode::Decode<'c, DB>,
    usize: ColumnIndex<DB::Row>,
{
    query
        .fetch_all(executor)
        .await
        .map_err(into_c3p0_error)?
        .iter()
        .map(|row| to_model(codec, row, 0, 1, 2, 3, 4))
        .collect::<Result<Vec<_>, C3p0Error>>()
}

#[inline]
pub async fn delete<'e, 'q: 'e, E, DB, DATA, DeleteQueryResult: ResultWithRowCount>(
    obj: Model<DATA>,
    executor: E,
    queries: &'q Queries,
) -> Result<Model<DATA>, C3p0Error>
where
    DB: Database<QueryResult = DeleteQueryResult>,
    <DB as sqlx::database::HasArguments<'q>>::Arguments: sqlx::IntoArguments<'q, DB>,
    E: Executor<'e, Database = DB>,
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send + Sync,
    for<'c> i32:
        sqlx::types::Type<DB> + sqlx::decode::Decode<'c, DB> + sqlx::encode::Encode<'c, DB>,
    for<'c> i64:
        sqlx::types::Type<DB> + sqlx::decode::Decode<'c, DB> + sqlx::encode::Encode<'c, DB>,
    for<'c> serde_json::value::Value:
        sqlx::types::Type<DB> + sqlx::decode::Decode<'c, DB> + sqlx::encode::Encode<'c, DB>,
    usize: ColumnIndex<DB::Row>,
{
    let result = sqlx::query(&queries.delete_sql_query)
        .bind(obj.id)
        .bind(obj.version)
        .execute(executor)
        .await
        .map_err(into_c3p0_error)?
        .rows_affected();

    if result == 0 {
        return Err(C3p0Error::OptimisticLockError {
            message: format!(
                "Cannot update data in table [{}] with id [{}], version [{}]: data was changed!",
                &queries.qualified_table_name, &obj.id, &obj.version
            ),
        });
    }

    Ok(obj)
}

#[inline]
pub async fn update<
    'e,
    'q: 'e,
    E,
    DB,
    DATA,
    CODEC: JsonCodec<DATA>,
    DeleteQueryResult: ResultWithRowCount,
>(
    obj: Model<DATA>,
    executor: E,
    queries: &'q Queries,
    codec: &CODEC,
) -> Result<Model<DATA>, C3p0Error>
where
    DB: Database<QueryResult = DeleteQueryResult>,
    <DB as sqlx::database::HasArguments<'q>>::Arguments: sqlx::IntoArguments<'q, DB>,
    E: Executor<'e, Database = DB>,
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send + Sync,
    for<'c> i32:
        sqlx::types::Type<DB> + sqlx::decode::Decode<'c, DB> + sqlx::encode::Encode<'c, DB>,
    for<'c> i64:
        sqlx::types::Type<DB> + sqlx::decode::Decode<'c, DB> + sqlx::encode::Encode<'c, DB>,
    for<'c> serde_json::value::Value:
        sqlx::types::Type<DB> + sqlx::decode::Decode<'c, DB> + sqlx::encode::Encode<'c, DB>,
    usize: ColumnIndex<DB::Row>,
{
    let json_data = codec.to_value(&obj.data)?;
    let previous_version = obj.version;
    let updated_model = obj.into_new_version(get_current_epoch_millis());
    
    let result = {
        sqlx::query(&queries.update_sql_query)
            .bind(updated_model.version)
            .bind(updated_model.update_epoch_millis)
            .bind(json_data)
            .bind(updated_model.id)
            .bind(previous_version)
            .execute(executor)
            .await
            .map_err(into_c3p0_error)
            .map(|done| done.rows_affected())?
    };

    if result == 0 {
        return Err(C3p0Error::OptimisticLockError {
            message: format!(
                "Cannot update data in table [{}] with id [{}], version [{}]: data was changed!",
                queries.qualified_table_name, &updated_model.id, &previous_version
            ),
        });
    }

    Ok(updated_model)
}
