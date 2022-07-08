use crate::utils::*;
use crate::*;
use c3p0::time::utils::get_current_epoch_millis;
use std::time::Duration;

#[test]
#[cfg(not(feature = "in_memory"))]
fn should_create_and_drop_table() -> Result<(), C3p0Error> {
    test(async {
        if db_specific::db_type() == DbType::InMemory {
            return Ok(());
        }

        let data = data(false).await;
        let pool = &data.0;

        let table_name = format!("TEST_TABLE_{}", rand_string(8));
        let jpo = &C3p0JsonBuilder::<C3p0Impl>::new(table_name).build();

        let model = NewModel::new(TestData {
            first_name: "my_first_name".to_owned(),
            last_name: "my_last_name".to_owned(),
        });

        pool.transaction::<_, C3p0Error, _, _>(|mut conn| async move {
            let conn = &mut conn;
            assert!(jpo.drop_table_if_exists(conn, false).await.is_ok());
            Ok(())
        })
        .await?;

        let model_clone = model.clone();
        pool.transaction::<_, C3p0Error, _, _>(|mut conn| async move {
            let conn = &mut conn;
            assert!(jpo.save(conn, model_clone).await.is_err());
            Ok(())
        })
        .await?;

        let model_clone = model.clone();
        pool.transaction::<_, C3p0Error, _, _>(|mut conn| async move {
            let conn = &mut conn;
            println!("first {:?}", jpo.create_table_if_not_exists(conn).await);

            assert!(jpo.create_table_if_not_exists(conn).await.is_ok());
            assert!(jpo.create_table_if_not_exists(conn).await.is_ok());

            assert!(jpo.save(conn, model_clone).await.is_ok());

            assert!(jpo.drop_table_if_exists(conn, false).await.is_ok());
            assert!(jpo.drop_table_if_exists(conn, true).await.is_ok());
            Ok(())
        })
        .await?;

        let model_clone = model.clone();
        pool.transaction::<_, C3p0Error, _, _>(|mut conn| async move {
            let conn = &mut conn;
            assert!(jpo.save(conn, model_clone).await.is_err());
            Ok(())
        })
        .await?;

        let model_clone = model.clone();
        pool.transaction::<_, C3p0Error, _, _>(|mut conn| async move {
            let conn = &mut conn;
            println!("second {:?}", jpo.create_table_if_not_exists(conn).await);

            assert!(jpo.create_table_if_not_exists(conn).await.is_ok());
            assert!(jpo.save(conn, model_clone).await.is_ok());
            Ok(())
        })
        .await?;

        Ok(())
    })
}

#[test]
fn basic_crud() -> Result<(), C3p0Error> {
    test(async {
        let data = data(false).await;
        let pool = &data.0;

        pool.transaction(|mut conn| async move {
            let conn = &mut conn;
            let table_name = format!("TEST_TABLE_{}", rand_string(8));
            let jpo = C3p0JsonBuilder::<C3p0Impl>::new(table_name).build();

            assert!(jpo.create_table_if_not_exists(conn).await.is_ok());
            jpo.delete_all(conn).await.unwrap();

            let model = NewModel::new(TestData {
                first_name: "my_first_name".to_owned(),
                last_name: "my_last_name".to_owned(),
            });

            let saved_model = jpo.save(conn, model.clone()).await.unwrap();
            assert!(saved_model.id >= 0);

            let found_model = jpo
                .fetch_one_optional_by_id(conn, &saved_model)
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

            let deleted = jpo.delete_by_id(conn, &saved_model).await.unwrap();
            assert_eq!(1, deleted);
            Ok(())
        })
        .await
    })
}

#[test]
fn should_fetch_all() -> Result<(), C3p0Error> {
    test(async {
        let data = data(false).await;
        let pool = &data.0;

        pool.transaction(|mut conn| async move {
            let conn = &mut conn;
            let table_name = format!("TEST_TABLE_{}", rand_string(8));
            let jpo = C3p0JsonBuilder::<C3p0Impl>::new(table_name).build();

            assert!(jpo.create_table_if_not_exists(conn).await.is_ok());
            jpo.delete_all(conn).await.unwrap();

            let model = NewModel::new(TestData {
                first_name: "my_first_name".to_owned(),
                last_name: "my_last_name".to_owned(),
            });

            let saved_model_0 = jpo.save(conn, model.clone()).await.unwrap();
            let saved_model_1 = jpo.save(conn, model.clone()).await.unwrap();
            let saved_model_2 = jpo.save(conn, model.clone()).await.unwrap();

            let models = jpo.fetch_all(conn).await.unwrap();

            assert_eq!(3, models.len());
            assert_eq!(saved_model_0.id, models[0].id);
            assert_eq!(saved_model_1.id, models[1].id);
            assert_eq!(saved_model_2.id, models[2].id);
            Ok(())
        })
        .await
    })
}

#[test]
fn should_delete_all() -> Result<(), C3p0Error> {
    test(async {
        let data = data(false).await;
        let pool = &data.0;

        pool.transaction(|mut conn| async move {
            let conn = &mut conn;
            let table_name = format!("TEST_TABLE_{}", rand_string(8));
            let jpo = C3p0JsonBuilder::<C3p0Impl>::new(table_name).build();

            assert!(jpo.drop_table_if_exists(conn, true).await.is_ok());
            assert!(jpo.create_table_if_not_exists(conn).await.is_ok());

            let model = NewModel::new(TestData {
                first_name: "my_first_name".to_owned(),
                last_name: "my_last_name".to_owned(),
            });

            let model1 = jpo.save(conn, model.clone()).await.unwrap();
            jpo.save(conn, model.clone()).await.unwrap();
            jpo.save(conn, model.clone()).await.unwrap();

            assert!(jpo.fetch_one_by_id(conn, &model1.id).await.is_ok());
            assert_eq!(1, jpo.delete_by_id(conn, &model1.id).await.unwrap());
            assert!(jpo.fetch_one_by_id(conn, &model1).await.is_err());
            assert_eq!(2, jpo.count_all(conn).await.unwrap());

            assert_eq!(2, jpo.delete_all(conn).await.unwrap());
            assert_eq!(0, jpo.count_all(conn).await.unwrap());
            Ok(())
        })
        .await
    })
}

#[test]
fn should_count() -> Result<(), C3p0Error> {
    test(async {
        let data = data(false).await;
        let pool = &data.0;

        pool.transaction(|mut conn| async move {
            let conn = &mut conn;
            let table_name = format!("TEST_TABLE_{}", rand_string(8));
            let jpo = C3p0JsonBuilder::<C3p0Impl>::new(table_name).build();

            assert!(jpo.create_table_if_not_exists(conn).await.is_ok());
            assert!(jpo.delete_all(conn).await.is_ok());

            assert_eq!(0, jpo.count_all(conn).await.unwrap());

            let model = NewModel::new(TestData {
                first_name: "my_first_name".to_owned(),
                last_name: "my_last_name".to_owned(),
            });

            jpo.save(conn, model.clone()).await.unwrap();
            assert_eq!(1, jpo.count_all(conn).await.unwrap());

            jpo.save(conn, model.clone()).await.unwrap();
            assert_eq!(2, jpo.count_all(conn).await.unwrap());

            jpo.save(conn, model.clone()).await.unwrap();
            assert_eq!(3, jpo.count_all(conn).await.unwrap());

            assert_eq!(3, jpo.delete_all(conn).await.unwrap());
            assert_eq!(0, jpo.count_all(conn).await.unwrap());

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

        pool.transaction(|mut conn| async move {
            let conn = &mut conn;
            let table_name = format!("TEST_TABLE_{}", rand_string(8));
            let jpo = C3p0JsonBuilder::<C3p0Impl>::new(table_name).build();

            assert!(jpo.create_table_if_not_exists(conn).await.is_ok());

            let model = NewModel::new(TestData {
                first_name: "my_first_name".to_owned(),
                last_name: "my_last_name".to_owned(),
            });

            let model = jpo.save(conn, model.clone()).await.unwrap();
            assert!(jpo.exists_by_id(conn, &model).await.unwrap());
            assert!(jpo.exists_by_id(conn, &model.id).await.unwrap());

            assert_eq!(1, jpo.delete_by_id(conn, &model).await.unwrap());
            assert!(!jpo.exists_by_id(conn, &model).await.unwrap());
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

        pool.transaction(|mut conn| async move {
            let conn = &mut conn;
            let table_name = format!("TEST_TABLE_{}", rand_string(8));
            let jpo = C3p0JsonBuilder::<C3p0Impl>::new(table_name).build();

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

            let found_model = jpo.fetch_one_by_id(conn, &saved_model).await.unwrap();
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

        pool.transaction(|mut conn| async move {
            let conn = &mut conn;
            let table_name = format!("TEST_TABLE_{}", rand_string(8));
            let jpo = C3p0JsonBuilder::<C3p0Impl>::new(&table_name).build();

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
                Ok(_) => assert!(false),
                Err(e) => match e {
                    C3p0Error::OptimisticLockError { message } => {
                        assert!(message.contains(&table_name));
                        assert!(message.contains(&format!(
                            "id [{}], version [{}]",
                            saved_model.id, saved_model.version
                        )));
                    }
                    _ => assert!(false),
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

        pool.transaction(|mut conn| async move {
            let conn = &mut conn;
            let table_name = format!("TEST_TABLE_{}", rand_string(8));
            let jpo = C3p0JsonBuilder::<C3p0Impl>::new(table_name).build();

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

            assert!(!jpo.exists_by_id(conn, &saved_model).await.unwrap());

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

        pool.transaction(|mut conn| async move {
            let conn = &mut conn;
            let table_name = format!("TEST_TABLE_{}", rand_string(8));
            let jpo = C3p0JsonBuilder::<C3p0Impl>::new(&table_name).build();

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
                Ok(_) => assert!(false),
                Err(e) => match e {
                    C3p0Error::OptimisticLockError { message } => {
                        assert!(message.contains(&table_name));
                        assert!(message.contains(&format!(
                            "id [{}], version [{}]",
                            saved_model.id, saved_model.version
                        )));
                    }
                    _ => assert!(false),
                },
            };

            assert!(jpo.exists_by_id(conn, &saved_model).await.unwrap());

            Ok(())
        })
        .await
    })
}

#[test]
fn json_should_perform_for_update_fetches() -> Result<(), C3p0Error> {
    test(async {
        let data = data(false).await;
        let pool = &data.0;
        let c3p0: C3p0Impl = pool.clone();
        let table_name = format!("TEST_TABLE_{}", rand_string(8));
        let jpo = &C3p0JsonBuilder::<C3p0Impl>::new(table_name).build();

        let model = NewModel::new(TestData {
            first_name: "my_first_name".to_owned(),
            last_name: "my_last_name".to_owned(),
        });

        let result: Result<_, C3p0Error> = c3p0
            .transaction(|mut conn| async move {
                let conn = &mut conn;
                assert!(jpo.create_table_if_not_exists(conn).await.is_ok());
                assert!(jpo.save(conn, model.clone()).await.is_ok());
                assert!(jpo.save(conn, model.clone()).await.is_ok());
                Ok(())
            })
            .await;
        assert!(result.is_ok());

        // fetch all ForUpdate::Default
        let result: Result<_, C3p0Error> = c3p0
            .transaction(|mut conn| async move {
                jpo.fetch_all_for_update(&mut conn, &ForUpdate::Default)
                    .await
            })
            .await;
        assert!(result.is_ok());

        if db_specific::db_type() != DbType::MySql && db_specific::db_type() != DbType::TiDB {
            // fetch one ForUpdate::NoWait
            let result: Result<_, C3p0Error> = c3p0
                .transaction(|mut conn| async move {
                    jpo.fetch_one_optional_by_id_for_update(&mut conn, &0, &ForUpdate::NoWait)
                        .await
                })
                .await;
            assert!(result.is_ok());

            // fetch one ForUpdate::SkipLocked
            let result: Result<_, C3p0Error> = c3p0
                .transaction(|mut conn| async move {
                    jpo.fetch_one_optional_by_id_for_update(&mut conn, &0, &ForUpdate::SkipLocked)
                        .await
                })
                .await;
            assert!(result.is_ok());
        }

        // fetch all ForUpdate::No
        let result: Result<_, C3p0Error> = c3p0
            .transaction(|mut conn| async move {
                jpo.fetch_all_for_update(&mut conn, &ForUpdate::No).await
            })
            .await;
        assert!(result.is_ok());

        {
            assert!(pool
                .transaction(
                    |mut conn| async move { jpo.drop_table_if_exists(&mut conn, true).await }
                )
                .await
                .is_ok());
        }
        Ok(())
    })
}
