use c3p0_pg_migrate::migration::Migration;
use c3p0_pg_migrate::{PgMigrateBuilder, C3P0_MIGRATE_TABLE_DEFAULT};
use testcontainers::clients;
use c3p0::pool::{C3p0Base, ConnectionBase};

mod shared;

#[test]
fn should_create_the_c3p0_migrate_table_with_default_name() -> Result<(), Box<std::error::Error>> {
    let docker = clients::Cli::default();
    let postgres_node = shared::new_connection(&docker);

    let pg_migrate = PgMigrateBuilder::new().with_migrations(vec![]).build();

    pg_migrate.migrate(&postgres_node.0)?;

    let conn = postgres_node.0.connection().unwrap();
    assert!(conn
        .fetch_all_values::<i64>(&format!("select count(*) from {}", C3P0_MIGRATE_TABLE_DEFAULT),
            &[]
        )
        .is_ok());

    Ok(())
}

#[test]
fn should_create_the_c3p0_migrate_table_with_custom_name() -> Result<(), Box<std::error::Error>> {
    let docker = clients::Cli::default();
    let postgres_node = shared::new_connection(&docker);

    let custom_name = "c3p0_custom_name";

    let pg_migrate = PgMigrateBuilder::new()
        .with_table_name(custom_name)
        .with_migrations(vec![])
        .build();

    pg_migrate.migrate(&postgres_node.0)?;

    let conn = postgres_node.0.connection().unwrap();
    assert!(conn
        .fetch_all_values::<i64>(&format!("select count(*) from {}", custom_name), &[])
        .is_ok());

    Ok(())
}

#[test]
fn should_execute_migrations() -> Result<(), Box<std::error::Error>> {
    let docker = clients::Cli::default();
    let postgres_node = shared::new_connection(&docker);
    let c3p0 = postgres_node.0.clone();

    let custom_name = "c3p0_custom_name";

    let pg_migrate = PgMigrateBuilder::new()
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
        .build();

    pg_migrate.migrate(&c3p0)?;

    let conn = postgres_node.0.connection().unwrap();

    assert!(conn
        .fetch_all_values::<i64>(&format!("select count(*) from {}", custom_name), &[])
        .is_ok());
    assert!(conn
        .fetch_all_values::<i64>(&format!("select count(*) from FIRST_TABLE"), &[])
        .is_ok());
    assert!(conn
        .fetch_all_values::<i64>(&format!("select count(*) from SECOND_TABLE"), &[])
        .is_ok());

    let status = pg_migrate.fetch_migrations_history(&conn).unwrap();
    assert_eq!(2, status.len());
    assert_eq!("first", status.get(0).unwrap().data.migration_id);

    Ok(())
}

#[test]
fn should_not_execute_same_migrations_twice() -> Result<(), Box<std::error::Error>> {
    let docker = clients::Cli::default();
    let postgres_node = shared::new_connection(&docker);
    let c3p0 = postgres_node.0.clone();

    let custom_name = "c3p0_custom_name";

    let pg_migrate = PgMigrateBuilder::new()
        .with_table_name(custom_name)
        .with_migrations(vec![Migration {
            id: "first".to_owned(),
            up: "create table FIRST_TABLE (id int)".to_owned(),
            down: "".to_owned(),
        }])
        .build();

    pg_migrate.migrate(&c3p0)?;
    pg_migrate.migrate(&c3p0)?;

    let conn = postgres_node.0.connection().unwrap();

    assert!(conn
        .fetch_all_values::<i64>(&format!("select count(*) from {}", custom_name), &[])
        .is_ok());
    assert!(conn
        .fetch_all_values::<i64>(&format!("select count(*) from FIRST_TABLE"), &[])
        .is_ok());

    let status = pg_migrate.fetch_migrations_history(&conn).unwrap();
    assert_eq!(1, status.len());
    assert_eq!("first", status.get(0).unwrap().data.migration_id);

    Ok(())
}

#[test]
fn should_handle_parallel_executions() -> Result<(), Box<std::error::Error>> {
    let docker = clients::Cli::default();
    let postgres_node = shared::new_connection(&docker);
    let c3p0 = postgres_node.0.clone();

    let custom_name = "c3p0_custom_name";
    let pg_migrate = PgMigrateBuilder::new()
        .with_table_name(custom_name)
        .with_migrations(vec![Migration {
            id: "first".to_owned(),
            up: "create table FIRST_TABLE (id int)".to_owned(),
            down: "".to_owned(),
        }])
        .build();

    let mut threads = vec![];

    for _i in 1..50 {
        let pool_clone = c3p0.clone();
        let pg_migrate = pg_migrate.clone();

        let handle = std::thread::spawn(move || {
            //println!("Thread [{:?}] - {} started", std::thread::current().id(), i);
            let result = pg_migrate.migrate(&pool_clone);
            //println!("Thread [{:?}] - {} completed: {:?}", std::thread::current().id(), i, result);
            assert!(result.is_ok());
        });
        threads.push(handle);
    }

    for handle in threads {
        handle.join().unwrap();
    }

    let status = pg_migrate
        .fetch_migrations_history(&c3p0.connection().unwrap())
        .unwrap();
    assert_eq!(1, status.len());
    assert_eq!("first", status.get(0).unwrap().data.migration_id);

    Ok(())
}
