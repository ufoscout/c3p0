use crate::utils::*;
use crate::*;
use c3p0::time::get_current_epoch_millis;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[test]
fn should_create_and_drop_table() -> Result<(), C3p0Error> {
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub struct TestData {
        pub first_name: String,
        pub last_name: String,
    }

    impl c3p0::DataType for TestData {
        const TABLE_NAME: &'static str =
            const_format::concatcp!("TEST_TABLE_", const_random::const_random!(u64));
        type CODEC = Self;
    }

    run_test(async {
        if [DbType::InMemory].contains(&db_specific::db_type()) {
            return Ok(());
        }

        let data = data(false).await;
        let pool = &data.0;

        let model = NewRecord::new(TestData {
            first_name: "my_first_name".to_owned(),
            last_name: "my_last_name".to_owned(),
        });

        pool.transaction::<_, C3p0Error, _>(async |conn| {
            assert!(conn.drop_table_if_exists::<TestData>(false).await.is_ok());
            Ok(())
        })
        .await?;

        let model_clone = model.clone();
        pool.transaction::<_, C3p0Error, _>(async |conn| {
            assert!(conn.save(model_clone).await.is_err());
            Ok(())
        })
        .await?;

        let model_clone = model.clone();
        pool.transaction::<_, C3p0Error, _>(async |conn| {
            println!(
                "first {:?}",
                conn.create_table_if_not_exists::<TestData>().await
            );

            assert!(conn.create_table_if_not_exists::<TestData>().await.is_ok());
            assert!(conn.create_table_if_not_exists::<TestData>().await.is_ok());

            assert!(conn.save(model_clone).await.is_ok());

            assert!(conn.drop_table_if_exists::<TestData>(false).await.is_ok());
            assert!(conn.drop_table_if_exists::<TestData>(true).await.is_ok());
            Ok(())
        })
        .await?;

        let model_clone = model.clone();
        pool.transaction::<_, C3p0Error, _>(async |conn| {
            assert!(conn.save(model_clone).await.is_err());
            Ok(())
        })
        .await?;

        let model_clone = model.clone();
        pool.transaction::<_, C3p0Error, _>(async |conn| {
            println!(
                "second {:?}",
                conn.create_table_if_not_exists::<TestData>().await
            );

            assert!(conn.create_table_if_not_exists::<TestData>().await.is_ok());
            assert!(conn.save(model_clone).await.is_ok());
            Ok(())
        })
        .await?;

        Ok(())
    })
}

#[test]
fn basic_crud() -> Result<(), C3p0Error> {
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub struct TestData {
        pub first_name: String,
        pub last_name: String,
    }

    impl c3p0::DataType for TestData {
        const TABLE_NAME: &'static str =
            const_format::concatcp!("TEST_TABLE_", const_random::const_random!(u64));
        type CODEC = Self;
    }

    run_test(async {
        let data = data(false).await;
        let pool = &data.0;

        pool.transaction(async |conn| {
            assert!(conn.create_table_if_not_exists::<TestData>().await.is_ok());
            conn.delete_all::<TestData>().await.unwrap();

            let model = NewRecord::new(TestData {
                first_name: "my_first_name".to_owned(),
                last_name: "my_last_name".to_owned(),
            });

            let saved_model = conn.save(model.clone()).await.unwrap();
            println!("saved_model {saved_model:?}");
            // assert!(saved_model.id >= 0);

            let found_model = conn
                .fetch_one_optional_by_id::<TestData>(saved_model.id)
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

            let deleted = conn.delete_by_id::<TestData>(saved_model.id).await.unwrap();
            assert_eq!(1, deleted);
            Ok(())
        })
        .await
    })
}

#[test]
fn should_fetch_all() -> Result<(), C3p0Error> {
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub struct TestData {
        pub first_name: String,
        pub last_name: String,
    }

    impl c3p0::DataType for TestData {
        const TABLE_NAME: &'static str =
            const_format::concatcp!("TEST_TABLE_", const_random::const_random!(u64));
        type CODEC = Self;
    }

    run_test(async {
        let data = data(false).await;
        let pool = &data.0;

        pool.transaction(async |conn| {
            assert!(conn.create_table_if_not_exists::<TestData>().await.is_ok());
            conn.delete_all::<TestData>().await.unwrap();

            let model = NewRecord::new(TestData {
                first_name: "my_first_name".to_owned(),
                last_name: "my_last_name".to_owned(),
            });

            let saved_model_0 = conn.save(model.clone()).await.unwrap();
            let saved_model_1 = conn.save(model.clone()).await.unwrap();
            let saved_model_2 = conn.save(model.clone()).await.unwrap();

            let models = conn.fetch_all::<TestData>().await.unwrap();

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
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub struct TestData {
        pub first_name: String,
        pub last_name: String,
    }

    impl c3p0::DataType for TestData {
        const TABLE_NAME: &'static str =
            const_format::concatcp!("TEST_TABLE_", const_random::const_random!(u64));
        type CODEC = Self;
    }

    run_test(async {
        let data = data(false).await;
        let pool = &data.0;

        pool.transaction(async |conn| {
            assert!(conn.create_table_if_not_exists::<TestData>().await.is_ok());

            let model = NewRecord::new(TestData {
                first_name: "my_first_name".to_owned(),
                last_name: "my_last_name".to_owned(),
            });

            let model1 = conn.save(model.clone()).await.unwrap();
            conn.save(model.clone()).await.unwrap();
            conn.save(model.clone()).await.unwrap();

            assert!(conn.fetch_one_by_id::<TestData>(model1.id).await.is_ok());
            assert_eq!(1, conn.delete_by_id::<TestData>(model1.id).await.unwrap());
            assert!(conn.fetch_one_by_id::<TestData>(model1.id).await.is_err());
            assert_eq!(2, conn.count_all::<TestData>().await.unwrap());

            assert_eq!(2, conn.delete_all::<TestData>().await.unwrap());
            assert_eq!(0, conn.count_all::<TestData>().await.unwrap());
            Ok(())
        })
        .await
    })
}

#[test]
fn should_count() -> Result<(), C3p0Error> {
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub struct TestData {
        pub first_name: String,
        pub last_name: String,
    }

    impl c3p0::DataType for TestData {
        const TABLE_NAME: &'static str =
            const_format::concatcp!("TEST_TABLE_", const_random::const_random!(u64));
        type CODEC = Self;
    }

    run_test(async {
        let data = data(false).await;
        let pool = &data.0;

        pool.transaction(async |conn| {
            assert!(conn.create_table_if_not_exists::<TestData>().await.is_ok());
            assert!(conn.delete_all::<TestData>().await.is_ok());

            assert_eq!(0, conn.count_all::<TestData>().await.unwrap());

            let model = NewRecord::new(TestData {
                first_name: "my_first_name".to_owned(),
                last_name: "my_last_name".to_owned(),
            });

            conn.save(model.clone()).await.unwrap();
            assert_eq!(1, conn.count_all::<TestData>().await.unwrap());

            conn.save(model.clone()).await.unwrap();
            assert_eq!(2, conn.count_all::<TestData>().await.unwrap());

            conn.save(model.clone()).await.unwrap();
            assert_eq!(3, conn.count_all::<TestData>().await.unwrap());

            assert_eq!(3, conn.delete_all::<TestData>().await.unwrap());
            assert_eq!(0, conn.count_all::<TestData>().await.unwrap());

            Ok(())
        })
        .await
    })
}

#[test]
fn should_return_whether_exists_by_id() -> Result<(), C3p0Error> {
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub struct TestData {
        pub first_name: String,
        pub last_name: String,
    }

    impl c3p0::DataType for TestData {
        const TABLE_NAME: &'static str =
            const_format::concatcp!("TEST_TABLE_", const_random::const_random!(u64));
        type CODEC = Self;
    }

    run_test(async {
        let data = data(false).await;
        let pool = &data.0;

        pool.transaction(async |conn| {
            assert!(conn.create_table_if_not_exists::<TestData>().await.is_ok());

            let model = NewRecord::new(TestData {
                first_name: "my_first_name".to_owned(),
                last_name: "my_last_name".to_owned(),
            });

            let model = conn.save(model.clone()).await.unwrap();
            assert!(conn.exists_by_id::<TestData>(model.id).await.unwrap());
            assert!(conn.exists_by_id::<TestData>(model.id).await.unwrap());

            assert_eq!(1, conn.delete_by_id::<TestData>(model.id).await.unwrap());
            assert!(!conn.exists_by_id::<TestData>(model.id).await.unwrap());
            assert!(!conn.exists_by_id::<TestData>(model.id).await.unwrap());
            Ok(())
        })
        .await
    })
}

#[test]
fn should_update_and_increase_version() -> Result<(), C3p0Error> {
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub struct TestData {
        pub first_name: String,
        pub last_name: String,
    }

    impl c3p0::DataType for TestData {
        const TABLE_NAME: &'static str =
            const_format::concatcp!("TEST_TABLE_", const_random::const_random!(u64));
        type CODEC = Self;
    }

    run_test(async {
        let data = data(false).await;
        let pool = &data.0;

        pool.transaction(async |conn| {
            assert!(conn.create_table_if_not_exists::<TestData>().await.is_ok());
            conn.delete_all::<TestData>().await.unwrap();

            let model = NewRecord::new(TestData {
                first_name: "my_first_name".to_owned(),
                last_name: "my_last_name".to_owned(),
            });

            let current_epoch = get_current_epoch_millis();
            let mut saved_model = conn.save(model.clone()).await.unwrap();

            assert!(saved_model.create_epoch_millis >= current_epoch);
            assert_eq!(
                saved_model.create_epoch_millis,
                saved_model.update_epoch_millis
            );

            tokio::time::sleep(Duration::from_millis(10)).await;

            "second_first_name".clone_into(&mut saved_model.data.first_name);
            let mut updated_model = conn.update(saved_model.clone()).await.unwrap();
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
            "second_last_name".clone_into(&mut updated_model.data.last_name);
            updated_model = conn.update(updated_model.clone()).await.unwrap();
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

            let found_model = conn
                .fetch_one_by_id::<TestData>(saved_model.id)
                .await
                .unwrap();
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
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub struct TestData {
        pub first_name: String,
        pub last_name: String,
    }

    impl c3p0::DataType for TestData {
        const TABLE_NAME: &'static str =
            const_format::concatcp!("TEST_TABLE_", const_random::const_random!(u64));
        type CODEC = Self;
    }

    run_test(async {
        let data = data(false).await;
        let pool = &data.0;

        pool.transaction(async |conn| {
            assert!(conn.create_table_if_not_exists::<TestData>().await.is_ok());
            conn.delete_all::<TestData>().await.unwrap();

            let model = NewRecord::new(TestData {
                first_name: "my_first_name".to_owned(),
                last_name: "my_last_name".to_owned(),
            });

            let mut saved_model = conn.save(model.clone()).await.unwrap();

            "second_first_name".clone_into(&mut saved_model.data.first_name);
            assert!(conn.update(saved_model.clone()).await.is_ok());

            let expected_error = conn.update(saved_model.clone()).await;
            assert!(expected_error.is_err());

            match expected_error {
                Ok(_) => panic!(),
                Err(e) => match e {
                    C3p0Error::OptimisticLockError { cause } => {
                        assert!(cause.contains(<TestData as c3p0::DataType>::TABLE_NAME));
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
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub struct TestData {
        pub first_name: String,
        pub last_name: String,
    }

    impl c3p0::DataType for TestData {
        const TABLE_NAME: &'static str =
            const_format::concatcp!("TEST_TABLE_", const_random::const_random!(u64));
        type CODEC = Self;
    }

    run_test(async {
        let data = data(false).await;
        let pool = &data.0;

        pool.transaction(async |conn| {
            assert!(conn.create_table_if_not_exists::<TestData>().await.is_ok());
            conn.delete_all::<TestData>().await.unwrap();

            let model = NewRecord::new(TestData {
                first_name: "my_first_name".to_owned(),
                last_name: "my_last_name".to_owned(),
            });

            let saved_model = conn.save(model.clone()).await.unwrap();

            let deleted = conn.delete(saved_model.clone()).await.unwrap();
            assert_eq!(saved_model.id, deleted.id);

            assert!(conn.delete(saved_model.clone()).await.is_err());

            assert!(!conn.exists_by_id::<TestData>(saved_model.id).await.unwrap());

            Ok(())
        })
        .await
    })
}

#[test]
fn delete_should_return_optimistic_lock_exception() -> Result<(), C3p0Error> {
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub struct TestData {
        pub first_name: String,
        pub last_name: String,
    }

    impl c3p0::DataType for TestData {
        const TABLE_NAME: &'static str =
            const_format::concatcp!("TEST_TABLE_", const_random::const_random!(u64));
        type CODEC = Self;
    }

    run_test(async {
        let data = data(false).await;
        let pool = &data.0;

        pool.transaction(async |conn| {
            assert!(conn.create_table_if_not_exists::<TestData>().await.is_ok());
            conn.delete_all::<TestData>().await.unwrap();

            let model = NewRecord::new(TestData {
                first_name: "my_first_name".to_owned(),
                last_name: "my_last_name".to_owned(),
            });

            let saved_model = conn.save(model.clone()).await.unwrap();
            assert!(conn.update(saved_model.clone()).await.is_ok());

            let expected_error = conn.delete(saved_model.clone()).await;
            assert!(expected_error.is_err());

            match expected_error {
                Ok(_) => panic!(),
                Err(e) => match e {
                    C3p0Error::OptimisticLockError { cause } => {
                        assert!(cause.contains(<TestData as c3p0::DataType>::TABLE_NAME));
                        assert!(cause.contains(&format!(
                            "id [{:?}], version [{}]",
                            saved_model.id, saved_model.version
                        )));
                    }
                    _ => panic!(),
                },
            };

            assert!(conn.exists_by_id::<TestData>(saved_model.id).await.unwrap());

            Ok(())
        })
        .await
    })
}
