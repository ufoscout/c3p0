use crate::{PgC3p0JsonBuilder, PostgresIdType};
use c3p0_common::{IdType, json::Queries};

pub fn build_pg_queries<Id: IdType, DbId: PostgresIdType>(
    json_builder: PgC3p0JsonBuilder<Id, DbId>,
) -> Queries {
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
        qualified_table_name
    );

    Queries {
        count_all_sql_query: format!("SELECT COUNT(*) FROM {}", qualified_table_name,),

        exists_by_id_sql_query: format!(
            "SELECT EXISTS (SELECT 1 FROM {} WHERE {} = $1)",
            qualified_table_name, json_builder.id_field_name,
        ),

        find_all_sql_query: format!(
            "{} ORDER BY {} ASC",
            find_base_sql_query, json_builder.id_field_name,
        ),

        find_by_id_sql_query: format!(
            "{} WHERE {} = $1 LIMIT 1",
            find_base_sql_query, json_builder.id_field_name,
        ),

        find_base_sql_query,

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
            "INSERT INTO {} ({}, {}, {}, {}) VALUES ($1, $2, $2, $3) RETURNING {}",
            qualified_table_name,
            json_builder.version_field_name,
            json_builder.create_epoch_millis_field_name,
            json_builder.update_epoch_millis_field_name,
            json_builder.data_field_name,
            json_builder.id_field_name
        ),

        save_sql_query_with_id: format!(
            "INSERT INTO {} ({}, {}, {}, {}, {}) VALUES ($1, $2, $2, $3, $4)",
            qualified_table_name,
            json_builder.version_field_name,
            json_builder.create_epoch_millis_field_name,
            json_builder.update_epoch_millis_field_name,
            json_builder.data_field_name,
            json_builder.id_field_name
        ),

        update_sql_query: format!(
            "UPDATE {} SET {} = $1, {} = $2, {} = $3  WHERE {} = $4 AND {} = $5",
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
                    {} {} primary key,
                    {} int not null,
                    {} bigint not null,
                    {} bigint not null,
                    {} JSONB
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
        create_epoch_millis_field_name: json_builder.create_epoch_millis_field_name,
        update_epoch_millis_field_name: json_builder.update_epoch_millis_field_name,

        data_field_name: json_builder.data_field_name,
        schema_name: json_builder.schema_name,
    }
}
