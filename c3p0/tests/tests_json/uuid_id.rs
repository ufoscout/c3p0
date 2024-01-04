use crate::utils::*;
use crate::*;
use c3p0::time::utils::get_current_epoch_millis;
use std::time::Duration;

#[test]
fn basic_crud() -> Result<(), C3p0Error> {
    test(async {
        let data = data(false).await;
        let pool = &data.0;

        pool.transaction(|conn| async {
            let table_name = format!("TEST_TABLE_{}", rand_string(8));
            let jpo = new_uuid_builder(&table_name).build();

            assert!(jpo.create_table_if_not_exists(conn).await.is_ok());
            jpo.delete_all(conn).await.unwrap();

            let model = NewModel::new(TestData {
                first_name: "my_first_name".to_owned(),
                last_name: "my_last_name".to_owned(),
            });

            let saved_model = jpo.save(conn, model.clone()).await.unwrap();
            println!("saved_model {:?}", saved_model);
            // assert!(saved_model.id >= 0);

            assert_eq!(1, jpo.count_all(conn).await.unwrap());
            println!("{:?}", jpo.fetch_all(conn).await.unwrap());

            let found_model = jpo
                .fetch_one_optional_by_id(conn, &saved_model.id)
                .await
                .unwrap()
                .unwrap();
            assert_eq!(saved_model.id, found_model.id);
            assert_eq!(saved_model.version, found_model.version);
            assert_eq!(
                saved_model.create_epoch_millis,
                found_model.create_epoch_millis
            );
            assert_eq!(
                saved_model.update_epoch_millis,
                found_model.update_epoch_millis
            );
            assert_eq!(saved_model.data.first_name, found_model.data.first_name);
            assert_eq!(saved_model.data.last_name, found_model.data.last_name);

            let deleted = jpo.delete_by_id(conn, &saved_model.id).await.unwrap();
            assert_eq!(1, deleted);
            Ok(())
        })
        .await
    })
}

#[test]
fn should_return_whether_exists_by_id() -> Result<(), C3p0Error> {
    test(async {
        let data = data(false).await;
        let pool = &data.0;

        pool.transaction(|conn| async {
            let table_name = format!("TEST_TABLE_{}", rand_string(8));
            let jpo = new_uuid_builder(&table_name).build();

            assert!(jpo.create_table_if_not_exists(conn).await.is_ok());

            let model = NewModel::new(TestData {
                first_name: "my_first_name".to_owned(),
                last_name: "my_last_name".to_owned(),
            });

            let model = jpo.save(conn, model.clone()).await.unwrap();
            assert!(jpo.exists_by_id(conn, &model.id).await.unwrap());
            assert!(jpo.exists_by_id(conn, &model.id).await.unwrap());

            assert_eq!(1, jpo.delete_by_id(conn, &model.id).await.unwrap());
            assert!(!jpo.exists_by_id(conn, &model.id).await.unwrap());
            assert!(!jpo.exists_by_id(conn, &model.id).await.unwrap());
            Ok(())
        })
        .await
    })
}

#[test]
fn should_update_and_increase_version() -> Result<(), C3p0Error> {
    test(async {
        let data = data(false).await;
        let pool = &data.0;

        pool.transaction(|conn| async {
            let table_name = format!("TEST_TABLE_{}", rand_string(8));
            let jpo = new_uuid_builder(&table_name).build();

            assert!(jpo.create_table_if_not_exists(conn).await.is_ok());
            jpo.delete_all(conn).await.unwrap();

            let model = NewModel::new(TestData {
                first_name: "my_first_name".to_owned(),
                last_name: "my_last_name".to_owned(),
            });

            let current_epoch = get_current_epoch_millis();
            let mut saved_model = jpo.save(conn, model.clone()).await.unwrap();

            assert!(saved_model.create_epoch_millis >= current_epoch);
            assert_eq!(
                saved_model.create_epoch_millis,
                saved_model.update_epoch_millis
            );

            tokio::time::sleep(Duration::from_millis(10)).await;

            saved_model.data.first_name = "second_first_name".to_owned();
            let mut updated_model = jpo.update(conn, saved_model.clone()).await.unwrap();
            assert_eq!(saved_model.id, updated_model.id);
            assert_eq!(saved_model.version + 1, updated_model.version);
            assert_eq!(
                saved_model.create_epoch_millis,
                updated_model.create_epoch_millis
            );
            assert!(updated_model.update_epoch_millis > updated_model.create_epoch_millis);
            assert_eq!("second_first_name", updated_model.data.first_name);
            assert_eq!("my_last_name", updated_model.data.last_name);

            tokio::time::sleep(Duration::from_millis(10)).await;

            let previous_update_epoch_millis = updated_model.update_epoch_millis;
            updated_model.data.last_name = "second_last_name".to_owned();
            updated_model = jpo.update(conn, updated_model.clone()).await.unwrap();
            assert_eq!(saved_model.id, updated_model.id);
            assert_eq!(saved_model.version + 2, updated_model.version);
            assert_eq!(
                saved_model.create_epoch_millis,
                updated_model.create_epoch_millis
            );
            assert!(updated_model.update_epoch_millis > updated_model.create_epoch_millis);
            assert_eq!(
                saved_model.create_epoch_millis,
                updated_model.create_epoch_millis
            );
            assert!(updated_model.update_epoch_millis > previous_update_epoch_millis);
            assert_eq!("second_first_name", updated_model.data.first_name);
            assert_eq!("second_last_name", updated_model.data.last_name);

            let found_model = jpo.fetch_one_by_id(conn, &saved_model.id).await.unwrap();
            assert_eq!(found_model.id, updated_model.id);
            assert_eq!(found_model.version, updated_model.version);
            assert_eq!(
                found_model.create_epoch_millis,
                updated_model.create_epoch_millis
            );
            assert_eq!(
                found_model.update_epoch_millis,
                updated_model.update_epoch_millis
            );
            assert_eq!(found_model.data, updated_model.data);
            Ok(())
        })
        .await
    })
}

#[test]
fn update_should_return_optimistic_lock_exception() -> Result<(), C3p0Error> {
    test(async {
        let data = data(false).await;
        let pool = &data.0;

        pool.transaction(|conn| async {
            let table_name = format!("TEST_TABLE_{}", rand_string(8));
            let jpo = new_uuid_builder(&table_name).build();

            assert!(jpo.create_table_if_not_exists(conn).await.is_ok());
            jpo.delete_all(conn).await.unwrap();

            let model = NewModel::new(TestData {
                first_name: "my_first_name".to_owned(),
                last_name: "my_last_name".to_owned(),
            });

            let mut saved_model = jpo.save(conn, model.clone()).await.unwrap();

            saved_model.data.first_name = "second_first_name".to_owned();
            assert!(jpo.update(conn, saved_model.clone()).await.is_ok());

            let expected_error = jpo.update(conn, saved_model.clone()).await;
            assert!(expected_error.is_err());

            match expected_error {
                Ok(_) => panic!(),
                Err(e) => match e {
                    C3p0Error::OptimisticLockError { cause } => {
                        assert!(cause.contains(&table_name));
                        println!("cause {}", cause);
                        assert!(cause.contains(&format!(
                            "id [{:?}], version [{}]",
                            saved_model.id, saved_model.version
                        )));
                    }
                    _ => panic!(),
                },
            };

            Ok(())
        })
        .await
    })
}

#[test]
fn should_delete_based_on_id_and_version() -> Result<(), C3p0Error> {
    test(async {
        let data = data(false).await;
        let pool = &data.0;

        pool.transaction(|conn| async {
            let table_name = format!("TEST_TABLE_{}", rand_string(8));
            let jpo = new_uuid_builder(&table_name).build();

            assert!(jpo.create_table_if_not_exists(conn).await.is_ok());
            jpo.delete_all(conn).await.unwrap();

            let model = NewModel::new(TestData {
                first_name: "my_first_name".to_owned(),
                last_name: "my_last_name".to_owned(),
            });

            let saved_model = jpo.save(conn, model.clone()).await.unwrap();

            let deleted = jpo.delete(conn, saved_model.clone()).await.unwrap();
            assert_eq!(saved_model.id, deleted.id);

            assert!(jpo.delete(conn, saved_model.clone()).await.is_err());

            assert!(!jpo.exists_by_id(conn, &saved_model.id).await.unwrap());

            Ok(())
        })
        .await
    })
}

#[test]
fn delete_should_return_optimistic_lock_exception() -> Result<(), C3p0Error> {
    test(async {
        let data = data(false).await;
        let pool = &data.0;

        pool.transaction(|conn| async {
            let table_name = format!("TEST_TABLE_{}", rand_string(8));
            let jpo = new_uuid_builder(&table_name).build();

            assert!(jpo.create_table_if_not_exists(conn).await.is_ok());
            jpo.delete_all(conn).await.unwrap();

            let model = NewModel::new(TestData {
                first_name: "my_first_name".to_owned(),
                last_name: "my_last_name".to_owned(),
            });

            let saved_model = jpo.save(conn, model.clone()).await.unwrap();
            assert!(jpo.update(conn, saved_model.clone()).await.is_ok());

            let expected_error = jpo.delete(conn, saved_model.clone()).await;
            assert!(expected_error.is_err());

            match expected_error {
                Ok(_) => panic!(),
                Err(e) => match e {
                    C3p0Error::OptimisticLockError { cause } => {
                        assert!(cause.contains(&table_name));
                        assert!(cause.contains(&format!(
                            "id [{:?}], version [{}]",
                            saved_model.id, saved_model.version
                        )));
                    }
                    _ => panic!(),
                },
            };

            assert!(jpo.exists_by_id(conn, &saved_model.id).await.unwrap());

            Ok(())
        })
        .await
    })
}
