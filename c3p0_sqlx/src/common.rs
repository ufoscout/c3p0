use crate::{Db, DbRow};
use c3p0_common::{JsonCodec, Model};
use sqlx::Row;

/*
pub fn to_value_mapper<T: FromSqlOwned>(row: &Row) -> Result<T, Box<dyn std::error::Error>> {
    Ok(row.try_get(0).map_err(|_| C3p0Error::ResultNotFoundError)?)
}
*/

/*
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
    <DB as HasArguments<'_>>::Arguments
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
*/

#[cfg(any(feature = "postgres"))]
pub fn build_pg_queries<C3P0>(
    json_builder: c3p0_common::C3p0JsonBuilder<C3P0>,
) -> c3p0_common::json::Queries {
    let qualified_table_name = match &json_builder.schema_name {
        Some(schema_name) => format!(r#"{}."{}""#, schema_name, json_builder.table_name),
        None => json_builder.table_name.clone(),
    };

    c3p0_common::json::Queries {
        count_all_sql_query: format!("SELECT COUNT(*) FROM {}", qualified_table_name,),

        exists_by_id_sql_query: format!(
            "SELECT EXISTS (SELECT 1 FROM {} WHERE {} = $1)",
            qualified_table_name, json_builder.id_field_name,
        ),

        find_all_sql_query: format!(
            "SELECT {}, {}, {} FROM {} ORDER BY {} ASC",
            json_builder.id_field_name,
            json_builder.version_field_name,
            json_builder.data_field_name,
            qualified_table_name,
            json_builder.id_field_name,
        ),

        find_by_id_sql_query: format!(
            "SELECT {}, {}, {} FROM {} WHERE {} = $1 LIMIT 1",
            json_builder.id_field_name,
            json_builder.version_field_name,
            json_builder.data_field_name,
            qualified_table_name,
            json_builder.id_field_name,
        ),

        delete_sql_query: format!(
            "DELETE FROM {} WHERE {} = $1 AND {} = $2",
            qualified_table_name, json_builder.id_field_name, json_builder.version_field_name,
        ),

        delete_all_sql_query: format!("DELETE FROM {}", qualified_table_name,),

        delete_by_id_sql_query: format!(
            "DELETE FROM {} WHERE {} = $1",
            qualified_table_name, json_builder.id_field_name,
        ),

        save_sql_query: format!(
            "INSERT INTO {} ({}, {}) VALUES ($1, $2) RETURNING {}",
            qualified_table_name,
            json_builder.version_field_name,
            json_builder.data_field_name,
            json_builder.id_field_name
        ),

        update_sql_query: format!(
            "UPDATE {} SET {} = $1, {} = $2 WHERE {} = $3 AND {} = $4",
            qualified_table_name,
            json_builder.version_field_name,
            json_builder.data_field_name,
            json_builder.id_field_name,
            json_builder.version_field_name,
        ),

        create_table_sql_query: format!(
            r#"
                CREATE TABLE IF NOT EXISTS {} (
                    {} bigserial primary key,
                    {} int not null,
                    {} JSONB
                )
                "#,
            qualified_table_name,
            json_builder.id_field_name,
            json_builder.version_field_name,
            json_builder.data_field_name
        ),

        drop_table_sql_query: format!("DROP TABLE IF EXISTS {}", qualified_table_name),
        drop_table_sql_query_cascade: format!(
            "DROP TABLE IF EXISTS {} CASCADE",
            qualified_table_name
        ),

        lock_table_sql_query: Some(format!(
            "LOCK TABLE {} IN ACCESS EXCLUSIVE MODE",
            qualified_table_name
        )),

        qualified_table_name,
        table_name: json_builder.table_name,
        id_field_name: json_builder.id_field_name,
        version_field_name: json_builder.version_field_name,
        data_field_name: json_builder.data_field_name,
        schema_name: json_builder.schema_name,
    }
}

#[cfg(any(feature = "mysql"))]
pub fn build_mysql_queries<C3P0>(
    json_builder: c3p0_common::C3p0JsonBuilder<C3P0>,
) -> c3p0_common::json::Queries {
    let qualified_table_name = match &json_builder.schema_name {
        Some(schema_name) => format!(r#"{}."{}""#, schema_name, json_builder.table_name),
        None => json_builder.table_name.clone(),
    };

    c3p0_common::json::Queries {
        count_all_sql_query: format!("SELECT COUNT(*) FROM {}", qualified_table_name,),

        exists_by_id_sql_query: format!(
            "SELECT EXISTS (SELECT 1 FROM {} WHERE {} = ?)",
            qualified_table_name, json_builder.id_field_name,
        ),

        find_all_sql_query: format!(
            "SELECT {}, {}, {} FROM {} ORDER BY {} ASC",
            json_builder.id_field_name,
            json_builder.version_field_name,
            json_builder.data_field_name,
            qualified_table_name,
            json_builder.id_field_name,
        ),

        find_by_id_sql_query: format!(
            "SELECT {}, {}, {} FROM {} WHERE {} = ? LIMIT 1",
            json_builder.id_field_name,
            json_builder.version_field_name,
            json_builder.data_field_name,
            qualified_table_name,
            json_builder.id_field_name,
        ),

        delete_sql_query: format!(
            "DELETE FROM {} WHERE {} = ? AND {} = ?",
            qualified_table_name, json_builder.id_field_name, json_builder.version_field_name,
        ),

        delete_all_sql_query: format!("DELETE FROM {}", qualified_table_name,),

        delete_by_id_sql_query: format!(
            "DELETE FROM {} WHERE {} = ?",
            qualified_table_name, json_builder.id_field_name,
        ),

        save_sql_query: format!(
            "INSERT INTO {} ({}, {}) VALUES (?, ?)",
            qualified_table_name, json_builder.version_field_name, json_builder.data_field_name
        ),

        update_sql_query: format!(
            "UPDATE {} SET {} = ?, {} = ? WHERE {} = ? AND {} = ?",
            qualified_table_name,
            json_builder.version_field_name,
            json_builder.data_field_name,
            json_builder.id_field_name,
            json_builder.version_field_name,
        ),

        create_table_sql_query: format!(
            r#"
                CREATE TABLE IF NOT EXISTS {} (
                    {} BIGINT primary key NOT NULL AUTO_INCREMENT,
                    {} int not null,
                    {} JSON
                )
                "#,
            qualified_table_name,
            json_builder.id_field_name,
            json_builder.version_field_name,
            json_builder.data_field_name
        ),

        drop_table_sql_query: format!("DROP TABLE IF EXISTS {}", qualified_table_name),
        drop_table_sql_query_cascade: format!(
            "DROP TABLE IF EXISTS {} CASCADE",
            qualified_table_name
        ),

        lock_table_sql_query: Some(format!("LOCK TABLES {} WRITE", qualified_table_name)),

        qualified_table_name,
        table_name: json_builder.table_name,
        id_field_name: json_builder.id_field_name,
        version_field_name: json_builder.version_field_name,
        data_field_name: json_builder.data_field_name,
        schema_name: json_builder.schema_name,
    }
}
