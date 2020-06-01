use c3p0_common::json::Queries;
use c3p0_common::{C3p0Error, C3p0JsonBuilder, JsonCodec, Model};
use core::fmt::Display;


pub fn into_c3p0_error(error: sqlx::Error) -> C3p0Error {
    C3p0Error::DbError {
        db: "sqlx",
        code: None,
        cause: format!("{}", &error),
    }
}
/*
pub fn to_value_mapper<T: FromSqlOwned>(row: &Row) -> Result<T, Box<dyn std::error::Error>> {
    Ok(row.try_get(0).map_err(|_| C3p0Error::ResultNotFoundError)?)
}
*/
/*
#[inline]
pub fn to_model<
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send,
    CODEC: JsonCodec<DATA>,
    IdIdx: RowIndex + Display,
    VersionIdx: RowIndex + Display,
    DataIdx: RowIndex + Display,
>(
    codec: &CODEC,
    row: &Row,
    id_index: IdIdx,
    version_index: VersionIdx,
    data_index: DataIdx,
) -> Result<Model<DATA>, Box<dyn std::error::Error>> {
    let id = get_or_error(&row, id_index)?;
    let version = get_or_error(&row, version_index)?;
    let data = codec.from_value(get_or_error(&row, data_index)?)?;
    Ok(Model { id, version, data })
}
*/
/*
#[inline]
pub fn get_or_error<'a, I: RowIndex + Display, T: FromSql<'a>>(
    row: &'a Row,
    index: I,
) -> Result<T, C3p0Error> {
    row.try_get(&index)
        .map_err(|err| C3p0Error::RowMapperError {
            cause: format!("Row contains no values for index {}. Err: {}", index, err),
        })
}
*/

pub fn build_pg_queries<C3P0>(json_builder: C3p0JsonBuilder<C3P0>) -> Queries {
    let qualified_table_name = match &json_builder.schema_name {
        Some(schema_name) => format!(r#"{}."{}""#, schema_name, json_builder.table_name),
        None => json_builder.table_name.clone(),
    };

    Queries {
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
