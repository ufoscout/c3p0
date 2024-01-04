use c3p0_common::IdType;

use crate::{SqliteIdGenerator, SqlxSqliteC3p0JsonBuilder};

pub fn build_sqlite_queries<
    Id: IdType,
    Generator: SqliteIdGenerator<Id>,
>(
    json_builder: SqlxSqliteC3p0JsonBuilder<Id, Generator>,
) -> c3p0_common::json::Queries {
    let qualified_table_name = match &json_builder.schema_name {
        Some(schema_name) => format!(r#"{}."{}""#, schema_name, json_builder.table_name),
        None => json_builder.table_name.clone(),
    };

    let find_base_sql_query = format!(
        "SELECT {}, {}, {}, {}, {} FROM {}",
        json_builder.id_field_name,
        json_builder.version_field_name,
        json_builder.create_epoch_millis_field_name,
        json_builder.update_epoch_millis_field_name,
        json_builder.data_field_name,
        qualified_table_name,
    );

    c3p0_common::json::Queries {
        count_all_sql_query: format!("SELECT COUNT(*) FROM {}", qualified_table_name,),

        exists_by_id_sql_query: format!(
            "SELECT EXISTS (SELECT 1 FROM {} WHERE {} = ?)",
            qualified_table_name, json_builder.id_field_name,
        ),

        find_all_sql_query: format!(
            "{} ORDER BY {} ASC",
            find_base_sql_query, json_builder.id_field_name,
        ),

        find_by_id_sql_query: format!(
            "{} WHERE {} = ? LIMIT 1",
            find_base_sql_query, json_builder.id_field_name,
        ),

        find_base_sql_query,

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
            "INSERT INTO {} ({}, {}, {}, {}) VALUES (?, ?, ?, ?)",
            qualified_table_name,
            json_builder.version_field_name,
            json_builder.create_epoch_millis_field_name,
            json_builder.update_epoch_millis_field_name,
            json_builder.data_field_name
        ),

        save_sql_query_with_id: format!(
            "INSERT INTO {} ({}, {}, {}, {}, {}) VALUES (?, ?, ?, ?, ?)",
            qualified_table_name,
            json_builder.version_field_name,
            json_builder.create_epoch_millis_field_name,
            json_builder.update_epoch_millis_field_name,
            json_builder.data_field_name,
            json_builder.id_field_name,
        ),

        update_sql_query: format!(
            "UPDATE {} SET {} = ?, {} = ?, {} = ? WHERE {} = ? AND {} = ?",
            qualified_table_name,
            json_builder.version_field_name,
            json_builder.update_epoch_millis_field_name,
            json_builder.data_field_name,
            json_builder.id_field_name,
            json_builder.version_field_name,
        ),

        create_table_sql_query: format!(
            r#"
                CREATE TABLE IF NOT EXISTS {} (
                    {} {},
                    {} integer not null,
                    {} integer not null,
                    {} integer not null,
                    {} JSON
                )
                "#,
            qualified_table_name,
            json_builder.id_field_name,
            json_builder.id_generator.create_statement_column_type(),
            json_builder.version_field_name,
            json_builder.create_epoch_millis_field_name,
            json_builder.update_epoch_millis_field_name,
            json_builder.data_field_name
        ),

        drop_table_sql_query: format!("DROP TABLE IF EXISTS {}", qualified_table_name),
        drop_table_sql_query_cascade: format!("DROP TABLE IF EXISTS {}", qualified_table_name),

        lock_table_sql_query: None,

        qualified_table_name,
        table_name: json_builder.table_name,
        id_field_name: json_builder.id_field_name,
        version_field_name: json_builder.version_field_name,
        create_epoch_millis_field_name: json_builder.create_epoch_millis_field_name,
        update_epoch_millis_field_name: json_builder.update_epoch_millis_field_name,
        data_field_name: json_builder.data_field_name,
        schema_name: json_builder.schema_name,
    }
}
