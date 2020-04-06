use testcontainers::clients;

use crate::utils::rand_string;
use crate::*;

#[tokio::test]
async fn should_create_the_c3p0_migrate_table_with_default_name() -> Result<(), C3p0Error> {
    let docker = clients::Cli::default();
    let node = new_connection(&docker).await;

    let migrate = C3p0MigrateBuilder::new(node.0.clone())
        .with_migrations(vec![])
        .build();

    migrate.migrate().await?;

    node.0
        .transaction(|mut conn| async move {
            let conn = &mut conn;
            assert!(conn
                .fetch_all_values::<i64>(
                    &format!("select count(*) from {}", C3P0_MIGRATE_TABLE_DEFAULT),
                    &[]
                )
                .await
                .is_ok());

            assert!(conn
                .execute(&format!(r"DROP TABLE {}", C3P0_MIGRATE_TABLE_DEFAULT), &[])
                .await
                .is_ok());
            Ok(())
        })
        .await
}

#[tokio::test]
async fn should_create_the_c3p0_migrate_table_with_custom_name() -> Result<(), C3p0Error> {
    let docker = clients::Cli::default();
    let node = new_connection(&docker).await;

    let custom_name = &format!("c3p0_custom_name_{}", rand_string(8));

    let migrate = C3p0MigrateBuilder::new(node.0.clone())
        .with_table_name(custom_name)
        .with_migrations(vec![])
        .build();

    migrate.migrate().await?;

    node.0
        .transaction(|mut conn| async move {
            let conn = &mut conn;
            assert!(conn
                .fetch_all_values::<i64>(&format!("select count(*) from {}", custom_name), &[])
                .await
                .is_ok());
            Ok(())
        })
        .await
}

#[tokio::test]
async fn should_execute_migrations() -> Result<(), C3p0Error> {
    let docker = clients::Cli::default();
    let node = new_connection(&docker).await;

    let migration_table_name = &format!("c3p0_custom_name_{}", rand_string(8));
    let first_table_name = &format!("first_table_{}", rand_string(8));
    let second_table_name = &format!("second_table_{}", rand_string(8));

    let migrate = C3p0MigrateBuilder::new(node.0.clone())
        .with_table_name(migration_table_name)
        .with_migrations(vec![
            Migration {
                id: "first".to_owned(),
                up: format!("create table {} (id int)", first_table_name),
                down: "".to_owned(),
            },
            Migration {
                id: "second".to_owned(),
                up: format!("create table {} (id int)", second_table_name),
                down: "".to_owned(),
            },
        ])
        .build();

    migrate.migrate().await?;

    node.0
        .transaction(|mut conn| async move {
            let conn = &mut conn;
            assert!(conn
                .fetch_all_values::<i64>(
                    &format!("select count(*) from {}", migration_table_name),
                    &[]
                )
                .await
                .is_ok());
            assert!(conn
                .fetch_all_values::<i64>(&format!("select count(*) from {}", first_table_name), &[])
                .await
                .is_ok());
            assert!(conn
                .fetch_all_values::<i64>(
                    &format!("select count(*) from {}", second_table_name),
                    &[]
                )
                .await
                .is_ok());

            let status = migrate.get_migrations_history(conn).await.unwrap();
            assert_eq!(3, status.len());
            assert_eq!(
                "C3P0_INIT_MIGRATION",
                status.get(0).unwrap().data.migration_id
            );
            assert_eq!("first", status.get(1).unwrap().data.migration_id);
            assert_eq!("second", status.get(2).unwrap().data.migration_id);

            Ok(())
        })
        .await
}

#[tokio::test]
async fn should_not_execute_same_migrations_twice() -> Result<(), C3p0Error> {
    let docker = clients::Cli::default();
    let node = new_connection(&docker).await;

    let migration_table_name = &format!("c3p0_custom_name_{}", rand_string(8));
    let first_table_name = &format!("first_table_{}", rand_string(8));

    let migrate = C3p0MigrateBuilder::new(node.0.clone())
        .with_table_name(migration_table_name)
        .with_migrations(vec![Migration {
            id: "first".to_owned(),
            up: format!("create table {} (id int)", first_table_name),
            down: "".to_owned(),
        }])
        .build();

    migrate.migrate().await?;
    migrate.migrate().await?;

    node.0
        .transaction(|mut conn| async move {
            let conn = &mut conn;
            assert!(conn
                .fetch_all_values::<i64>(
                    &format!("select count(*) from {}", migration_table_name),
                    &[]
                )
                .await
                .is_ok());
            assert!(conn
                .fetch_all_values::<i64>(&format!("select count(*) from {}", first_table_name), &[])
                .await
                .is_ok());

            let status = migrate.get_migrations_history(conn).await.unwrap();
            assert_eq!(2, status.len());
            assert_eq!(
                "C3P0_INIT_MIGRATION",
                status.get(0).unwrap().data.migration_id
            );
            assert_eq!("first", status.get(1).unwrap().data.migration_id);

            Ok(())
        })
        .await
}

/*
#[tokio::test]
async fn should_handle_parallel_executions() -> Result<(), C3p0Error> {
    if db_specific::db_type() != crate::utils::DbType::Pg {
        return Ok(());
    }

    let docker = clients::Cli::default();
    let node = new_connection(&docker).await;
    let c3p0 = node.0.clone();

    let migration_table_name = &format!("c3p0_custom_name_{}", rand_string(8));
    let first_table_name = &format!("first_table_{}", rand_string(8));

    let migrate = std::sync::Arc::new(
        C3p0MigrateBuilder::new(c3p0.clone())
            .with_table_name(migration_table_name)
            .with_migrations(vec![Migration {
                id: "first".to_owned(),
                up: format!("create table {} (id int)", first_table_name),
                down: "".to_owned(),
            }])
            .build(),
    );

    let mut threads = vec![];

    for _i in 1..50 {
        let migrate = migrate.clone();

        let handle = tokio::spawn(async move {
            //println!("Thread [{:?}] - {} started", std::thread::current().id(), i);
            let result = migrate.migrate().await;
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
        let result = tokio::join!(handle);
        println!("thread result: \n{:?}", result);
        result.0.unwrap();
    }

    node.0
        .transaction(|mut conn| async move {
            let conn = &mut conn;
            let status = migrate.get_migrations_history(conn).await.unwrap();
            assert_eq!(2, status.len());
            assert_eq!(
                "C3P0_INIT_MIGRATION",
                status.get(0).unwrap().data.migration_id
            );
            assert_eq!("first", status.get(1).unwrap().data.migration_id);

            Ok(())
        })
        .await
}
*/

#[tokio::test]
async fn should_read_migrations_from_files() -> Result<(), C3p0Error> {
    let docker = clients::Cli::default();
    let node = new_connection(&docker).await;
    let c3p0 = node.0.clone();

    let migrate = C3p0MigrateBuilder::new(c3p0.clone())
        .with_migrations(from_fs("./tests/migrations_00")?)
        .build();

    migrate.migrate().await?;
    migrate.migrate().await?;

    node.0
        .transaction(|mut conn| async move {
            let conn = &mut conn;

            assert_eq!(
                3,
                conn.fetch_one_value::<i64>(&format!("select count(*) from TEST_TABLE"), &[])
                    .await?
            );

            let status = migrate.get_migrations_history(conn).await.unwrap();
            assert_eq!(3, status.len());
            assert_eq!("C3P0_INIT_MIGRATION", status[0].data.migration_id);
            assert_eq!("00010_create_test_data", status[1].data.migration_id);
            assert_eq!("00011_insert_test_data", status[2].data.migration_id);

            Ok(())
        })
        .await
}

const MIGRATIONS: include_dir::Dir = include_dir::include_dir!("./tests/migrations_00");

#[tokio::test]
async fn should_read_embedded_migrations() -> Result<(), C3p0Error> {
    let docker = clients::Cli::default();
    let node = new_connection(&docker).await;
    let c3p0 = node.0.clone();

    let migrate = C3p0MigrateBuilder::new(c3p0.clone())
        .with_migrations(from_embed(&MIGRATIONS).unwrap())
        .build();

    migrate.migrate().await?;
    migrate.migrate().await?;

    node.0
        .transaction(|mut conn| async move {
            assert_eq!(
                3,
                conn.fetch_one_value::<i64>(&format!("select count(*) from TEST_TABLE"), &[])
                    .await?
            );

            let status = migrate.get_migrations_history(&mut conn).await.unwrap();
            assert_eq!(3, status.len());
            assert_eq!("C3P0_INIT_MIGRATION", status[0].data.migration_id);
            assert_eq!("00010_create_test_data", status[1].data.migration_id);
            assert_eq!("00011_insert_test_data", status[2].data.migration_id);

            Ok(())
        })
        .await
}
