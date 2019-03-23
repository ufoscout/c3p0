use c3p0_pg_migrate::migration::Migration;
use c3p0_pg_migrate::{PgMigrateBuilder, C3P0_MIGRATE_TABLE_DEFAULT};
use testcontainers::clients;

mod shared;

#[test]
fn should_create_the_c3p0_migrate_table_with_default_name() -> Result<(), Box<std::error::Error>> {
    let docker = clients::Cli::default();
    let postgres_node = shared::new_connection(&docker);
    let conn = postgres_node.0.get().unwrap();

    let pg_migrate = PgMigrateBuilder::new().with_migrations(vec![]).build();

    pg_migrate.migrate(&conn)?;

    assert!(conn
        .execute(
            &format!("select * from {}", C3P0_MIGRATE_TABLE_DEFAULT),
            &[]
        )
        .is_ok());

    Ok(())
}

#[test]
fn should_create_the_c3p0_migrate_table_with_custom_name() -> Result<(), Box<std::error::Error>> {
    let docker = clients::Cli::default();
    let postgres_node = shared::new_connection(&docker);
    let conn = postgres_node.0.get().unwrap();

    let custom_name = "c3p0_custom_name";

    let pg_migrate = PgMigrateBuilder::new()
        .with_table_name(custom_name)
        .with_migrations(vec![])
        .build();

    pg_migrate.migrate(&conn)?;

    assert!(conn
        .execute(&format!("select * from {}", custom_name), &[])
        .is_ok());

    Ok(())
}

#[test]
fn should_execute_migrations() -> Result<(), Box<std::error::Error>> {
    let docker = clients::Cli::default();
    let postgres_node = shared::new_connection(&docker);
    let conn = postgres_node.0.get().unwrap();

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

    pg_migrate.migrate(&conn)?;

    assert!(conn
        .execute(&format!("select * from {}", custom_name), &[])
        .is_ok());
    assert!(conn
        .execute(&format!("select * from FIRST_TABLE"), &[])
        .is_ok());
    assert!(conn
        .execute(&format!("select * from SECOND_TABLE"), &[])
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
    let conn = postgres_node.0.get().unwrap();

    let custom_name = "c3p0_custom_name";

    let pg_migrate = PgMigrateBuilder::new()
        .with_table_name(custom_name)
        .with_migrations(vec![Migration {
            id: "first".to_owned(),
            up: "create table FIRST_TABLE (id int)".to_owned(),
            down: "".to_owned(),
        }])
        .build();

    pg_migrate.migrate(&conn)?;
    pg_migrate.migrate(&conn)?;

    assert!(conn
        .execute(&format!("select * from {}", custom_name), &[])
        .is_ok());
    assert!(conn
        .execute(&format!("select * from FIRST_TABLE"), &[])
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
    let pool = postgres_node.0;

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
        let pool_clone = pool.clone();
        let pg_migrate = pg_migrate.clone();

        let handle = std::thread::spawn(move || {
            //println!("Thread [{:?}] - {} started", std::thread::current().id(), i);
            let result = pg_migrate.migrate(&pool_clone.get().unwrap());
            //println!("Thread [{:?}] - {} completed: {:?}", std::thread::current().id(), i, result);
            assert!(result.is_ok());
        });
        threads.push(handle);
    }

    for handle in threads {
        handle.join().unwrap();
    }

    let status = pg_migrate
        .fetch_migrations_history(&pool.get().unwrap())
        .unwrap();
    assert_eq!(1, status.len());
    assert_eq!("first", status.get(0).unwrap().data.migration_id);

    Ok(())
}
