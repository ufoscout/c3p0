use c3p0_pg_migrate::{PgMigrateBuilder, C3P0_MIGRATE_TABLE_DEFAULT};
use testcontainers::clients;
use c3p0_pg_migrate::migration::Migration;

mod shared;

#[test]
fn should_create_the_c3p0_migrate_table_with_default_name() {
    let docker = clients::Cli::default();
    let postgres_node = shared::new_connection(&docker);
    let conn = postgres_node.0;

    let pg_migrate = PgMigrateBuilder::new()
        .with_migrations(vec![])
        .build();

    pg_migrate.migrate(&conn);

    assert!(conn.execute(&format!("select * from {}", C3P0_MIGRATE_TABLE_DEFAULT), &[] ).is_ok());
}

#[test]
fn should_create_the_c3p0_migrate_table_with_custom_name() {
    let docker = clients::Cli::default();
    let postgres_node = shared::new_connection(&docker);
    let conn = postgres_node.0;

    let custom_name = "c3p0_custom_name";

    let pg_migrate = PgMigrateBuilder::new()
        .with_table_name(custom_name)
        .with_migrations(vec![])
        .build();

    pg_migrate.migrate(&conn);

    assert!(conn.execute(&format!("select * from {}", custom_name), &[] ).is_ok());
}

#[test]
fn should_execute_migrations() {
    let docker = clients::Cli::default();
    let postgres_node = shared::new_connection(&docker);
    let conn = postgres_node.0;

    let custom_name = "c3p0_custom_name";

    let pg_migrate = PgMigrateBuilder::new()
        .with_table_name(custom_name)
        .with_migrations(vec![
            Migration{
                id: "first".to_owned(),
                up: "create table FIRST_TABLE (id int)".to_owned(),
                down: "".to_owned(),
            },
            Migration{
                id: "second".to_owned(),
                up: "create table SECOND_TABLE (id int)".to_owned(),
                down: "".to_owned(),
            }
        ])
        .build();

    pg_migrate.migrate(&conn);

    assert!(conn.execute(&format!("select * from {}", custom_name), &[] ).is_ok());
    assert!(conn.execute(&format!("select * from FIRST_TABLE"), &[] ).is_ok());
    assert!(conn.execute(&format!("select * from SECOND_TABLE"), &[] ).is_ok());

    let status = pg_migrate.migrations_status(&conn);
    assert_eq!(2, status.len());
    assert_eq!("first", status.get(0).unwrap().data.migration_id);
}