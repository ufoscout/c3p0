use c3p0_json::*;
use c3p0_migrate::migration::{fs::from_fs, Migration};
use c3p0_migrate::{C3p0MigrateBuilder, C3P0_MIGRATE_TABLE_DEFAULT};
use testcontainers::clients;

#[cfg(feature = "pg")]
mod shared_pg;
#[cfg(feature = "pg")]
use crate::shared_pg::*;

#[cfg(feature = "mysql")]
mod shared_mysql;
#[cfg(feature = "mysql")]
use crate::shared_mysql::*;

#[cfg(feature = "sqlite")]
mod shared_sqlite;
#[cfg(feature = "sqlite")]
use crate::shared_sqlite::*;

#[test]
fn should_create_the_c3p0_migrate_table_with_default_name() -> Result<(), Box<std::error::Error>> {
    let docker = clients::Cli::default();
    let node = new_connection(&docker);

    let migrate = C3p0MigrateBuilder::new().with_migrations(vec![]).build(node.0.clone());

    migrate.migrate()?;

    let conn = node.0.connection().unwrap();
    assert!(conn
        .fetch_all_values::<i64>(
            &format!("select count(*) from {}", C3P0_MIGRATE_TABLE_DEFAULT),
            &[]
        )
        .is_ok());

    Ok(())
}

#[test]
fn should_create_the_c3p0_migrate_table_with_custom_name() -> Result<(), Box<std::error::Error>> {
    let docker = clients::Cli::default();
    let node = new_connection(&docker);

    let custom_name = "c3p0_custom_name";

    let migrate = C3p0MigrateBuilder::new()
        .with_table_name(custom_name)
        .with_migrations(vec![])
        .build(node.0.clone());

    migrate.migrate()?;

    let conn = node.0.connection().unwrap();
    assert!(conn
        .fetch_all_values::<i64>(&format!("select count(*) from {}", custom_name), &[])
        .is_ok());

    Ok(())
}

#[test]
fn should_execute_migrations() -> Result<(), Box<std::error::Error>> {
    let docker = clients::Cli::default();
    let node = new_connection(&docker);

    let custom_name = "c3p0_custom_name";

    let migrate = C3p0MigrateBuilder::new()
        .with_table_name(custom_name)
        .with_migrations(vec![
            Migration {
                id: "first".to_owned(),
                up: "create table FIRST_TABLE (id int)".to_owned(),
                down: "".to_owned(),
            },
            Migration {
                id: "second".to_owned(),
                up: "create table SECOND_TABLE (id int)".to_owned(),
                down: "".to_owned(),
            },
        ])
        .build(node.0.clone());

    migrate.migrate()?;

    let conn = node.0.connection().unwrap();

    assert!(conn
        .fetch_all_values::<i64>(&format!("select count(*) from {}", custom_name), &[])
        .is_ok());
    assert!(conn
        .fetch_all_values::<i64>(&format!("select count(*) from FIRST_TABLE"), &[])
        .is_ok());
    assert!(conn
        .fetch_all_values::<i64>(&format!("select count(*) from SECOND_TABLE"), &[])
        .is_ok());

    let status = migrate.get_migrations_history(&conn).unwrap();
    assert_eq!(3, status.len());
    assert_eq!(
        "C3P0_INIT_MIGRATION",
        status.get(0).unwrap().data.migration_id
    );
    assert_eq!("first", status.get(1).unwrap().data.migration_id);
    assert_eq!("second", status.get(2).unwrap().data.migration_id);

    Ok(())
}

#[test]
fn should_not_execute_same_migrations_twice() -> Result<(), Box<std::error::Error>> {
    let docker = clients::Cli::default();
    let node = new_connection(&docker);

    let custom_name = "c3p0_custom_name";

    let migrate = C3p0MigrateBuilder::new()
        .with_table_name(custom_name)
        .with_migrations(vec![Migration {
            id: "first".to_owned(),
            up: "create table FIRST_TABLE (id int)".to_owned(),
            down: "".to_owned(),
        }])
        .build(node.0.clone());

    migrate.migrate()?;
    migrate.migrate()?;

    let conn = node.0.connection().unwrap();

    assert!(conn
        .fetch_all_values::<i64>(&format!("select count(*) from {}", custom_name), &[])
        .is_ok());
    assert!(conn
        .fetch_all_values::<i64>(&format!("select count(*) from FIRST_TABLE"), &[])
        .is_ok());

    let status = migrate.get_migrations_history(&conn).unwrap();
    assert_eq!(2, status.len());
    assert_eq!(
        "C3P0_INIT_MIGRATION",
        status.get(0).unwrap().data.migration_id
    );
    assert_eq!("first", status.get(1).unwrap().data.migration_id);

    Ok(())
}

#[cfg(feature = "pg")]
#[test]
fn should_handle_parallel_executions() -> Result<(), Box<std::error::Error>> {
    let docker = clients::Cli::default();
    let node = new_connection(&docker);
    let c3p0 = node.0.clone();

    let custom_name = "c3p0_custom_name";
    let migrate = C3p0MigrateBuilder::new()
        .with_table_name(custom_name)
        .with_migrations(vec![Migration {
            id: "first".to_owned(),
            up: "create table FIRST_TABLE (id int)".to_owned(),
            down: "".to_owned(),
        }])
        .build(c3p0.clone());

    let mut threads = vec![];

    for _i in 1..50 {
        let migrate = migrate.clone();

        let handle = std::thread::spawn(move || {
            //println!("Thread [{:?}] - {} started", std::thread::current().id(), i);
            let result = migrate.migrate();
            println!(
                "Thread [{:?}] - completed: {:?}",
                std::thread::current().id(),
                result
            );
            assert!(result.is_ok());
        });
        threads.push(handle);
    }

    for handle in threads {
        let result = handle.join();
        println!("thread result: \n{:?}", result);
        result.unwrap();
    }

    let status = migrate
        .get_migrations_history(&c3p0.connection().unwrap())
        .unwrap();
    assert_eq!(2, status.len());
    assert_eq!(
        "C3P0_INIT_MIGRATION",
        status.get(0).unwrap().data.migration_id
    );
    assert_eq!("first", status.get(1).unwrap().data.migration_id);

    Ok(())
}

#[test]
fn should_read_migrations_from_files() -> Result<(), Box<std::error::Error>> {
    let docker = clients::Cli::default();
    let node = new_connection(&docker);
    let c3p0 = node.0.clone();

    let migrate = C3p0MigrateBuilder::new()
        .with_migrations(from_fs("./tests/migrations_00")?)
        .build(c3p0.clone());

    migrate.migrate()?;
    migrate.migrate()?;

    let conn = node.0.connection().unwrap();

    assert_eq!(
        3,
        conn.fetch_one_value::<i64>(&format!("select count(*) from TEST_TABLE"), &[])?
    );

    let status = migrate.get_migrations_history(&conn).unwrap();
    assert_eq!(3, status.len());
    assert_eq!("C3P0_INIT_MIGRATION", status[0].data.migration_id);
    assert_eq!("00010_create_test_data", status[1].data.migration_id);
    assert_eq!("00011_insert_test_data", status[2].data.migration_id);

    Ok(())
}
