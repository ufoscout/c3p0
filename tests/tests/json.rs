use crate::utils::*;
use crate::*;
use chrono::Utc;
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

            let found_model = conn
                .fetch_one_optional_by_id::<TestData>(saved_model.id)
                .await
                .unwrap()
                .unwrap();
            assert_eq!(saved_model.id, found_model.id);
            assert_eq!(saved_model.version, found_model.version);
            assert_eq!(saved_model.create_time, found_model.create_time);
            assert_eq!(saved_model.update_time, found_model.update_time);
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

            let models = conn.fetch_all::<TestData>(0, None).await.unwrap();

            assert_eq!(3, models.len());
            assert_eq!(saved_model_0.id, models[0].id);
            assert_eq!(saved_model_1.id, models[1].id);
            assert_eq!(saved_model_2.id, models[2].id);

            let limited = conn.fetch_all::<TestData>(0, Some(2)).await.unwrap();
            assert_eq!(2, limited.len());
            assert_eq!(saved_model_0.id, limited[0].id);
            assert_eq!(saved_model_1.id, limited[1].id);

            let offset_only = conn.fetch_all::<TestData>(1, None).await.unwrap();
            assert_eq!(2, offset_only.len());
            assert_eq!(saved_model_1.id, offset_only[0].id);
            assert_eq!(saved_model_2.id, offset_only[1].id);

            let paged = conn.fetch_all::<TestData>(1, Some(1)).await.unwrap();
            assert_eq!(1, paged.len());
            assert_eq!(saved_model_1.id, paged[0].id);

            let past_end = conn.fetch_all::<TestData>(10, None).await.unwrap();
            assert!(past_end.is_empty());
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

        let before_save = Utc::now() - Duration::from_secs(1);

        let saved_model: Record<TestData> = pool
            .transaction::<_, C3p0Error, _>(async |conn| {
                assert!(conn.create_table_if_not_exists::<TestData>().await.is_ok());
                conn.delete_all::<TestData>().await.unwrap();

                let model = NewRecord::new(TestData {
                    first_name: "my_first_name".to_owned(),
                    last_name: "my_last_name".to_owned(),
                });

                let saved_model = conn.save(model).await.unwrap();
                assert!(saved_model.create_time >= before_save);
                assert_eq!(saved_model.create_time, saved_model.update_time);
                Ok(saved_model)
            })
            .await?;

        tokio::time::sleep(Duration::from_millis(10)).await;

        let updated_model: Record<TestData> = pool
            .transaction::<_, C3p0Error, _>(async |conn| {
                let mut to_update = saved_model.clone();
                "second_first_name".clone_into(&mut to_update.data.first_name);
                let updated_model = conn.update(to_update).await.unwrap();
                assert_eq!(saved_model.id, updated_model.id);
                assert_eq!(saved_model.version + 1, updated_model.version);
                assert_eq!(saved_model.create_time, updated_model.create_time);
                assert!(updated_model.update_time > updated_model.create_time);
                assert_eq!("second_first_name", updated_model.data.first_name);
                assert_eq!("my_last_name", updated_model.data.last_name);
                Ok(updated_model)
            })
            .await?;

        tokio::time::sleep(Duration::from_millis(10)).await;

        let previous_update_time = updated_model.update_time;
        let updated_model: Record<TestData> = pool
            .transaction::<_, C3p0Error, _>(async |conn| {
                let mut to_update = updated_model.clone();
                "second_last_name".clone_into(&mut to_update.data.last_name);
                let updated_model = conn.update(to_update).await.unwrap();
                assert_eq!(saved_model.id, updated_model.id);
                assert_eq!(saved_model.version + 2, updated_model.version);
                assert_eq!(saved_model.create_time, updated_model.create_time);
                assert!(updated_model.update_time > updated_model.create_time);
                assert!(updated_model.update_time > previous_update_time);
                assert_eq!("second_first_name", updated_model.data.first_name);
                assert_eq!("second_last_name", updated_model.data.last_name);
                Ok(updated_model)
            })
            .await?;

        pool.transaction(async |conn| {
            let found_model = conn
                .fetch_one_by_id::<TestData>(saved_model.id)
                .await
                .unwrap();
            assert_eq!(found_model.id, updated_model.id);
            assert_eq!(found_model.version, updated_model.version);
            assert_eq!(found_model.create_time, updated_model.create_time);
            assert_eq!(found_model.update_time, updated_model.update_time);
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

#[test]
fn query_with_tail_should_filter_order_and_limit() -> Result<(), C3p0Error> {
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub struct TestData {
        pub name: String,
        pub rank: i64,
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

        let saved_ids: Vec<i64> = pool
            .transaction::<_, C3p0Error, _>(async |conn| {
                conn.create_table_if_not_exists::<TestData>().await?;
                conn.delete_all::<TestData>().await?;
                let mut ids = Vec::new();
                for (name, rank) in [("alice", 30_i64), ("bob", 10), ("carol", 20)] {
                    let saved = conn
                        .save(NewRecord::new(TestData {
                            name: name.to_owned(),
                            rank,
                        }))
                        .await?;
                    ids.push(saved.id);
                }
                Ok(ids)
            })
            .await?;

        // 1. ORDER BY id ASC — exercises a tail with no placeholders.
        pool.transaction::<_, C3p0Error, _>(async |conn| {
            let rows = Record::<TestData>::query_with_tail("ORDER BY id ASC")
                .fetch_all(conn)
                .await?;
            assert_eq!(rows.len(), 3);
            assert_eq!(rows[0].data.name, "alice");
            assert_eq!(rows[1].data.name, "bob");
            assert_eq!(rows[2].data.name, "carol");
            Ok(())
        })
        .await?;

        // 2. ORDER BY DESC + LIMIT — exercises the slicing path.
        pool.transaction::<_, C3p0Error, _>(async |conn| {
            let rows = Record::<TestData>::query_with_tail("ORDER BY id DESC LIMIT 2")
                .fetch_all(conn)
                .await?;
            assert_eq!(rows.len(), 2);
            assert_eq!(rows[0].data.name, "carol");
            assert_eq!(rows[1].data.name, "bob");
            Ok(())
        })
        .await?;

        // 3. WHERE with bind — exercises the per-backend placeholder + .bind().
        // Placeholder syntax differs per dialect; bind type is `i64` everywhere
        // (id is i64 in c3p0 and the underlying column is signed BIGINT/INTEGER
        // on every supported backend).
        pool.transaction::<_, C3p0Error, _>(async |conn| {
            let target_id = saved_ids[1];
            let placeholder = match db_specific::db_type() {
                DbType::Pg => "$1",
                _ => "?",
            };
            let tail = format!("WHERE id = {placeholder}");
            let row = Record::<TestData>::query_with_tail(&tail)
                .bind(target_id)
                .fetch_one(conn)
                .await?;

            assert_eq!(row.data.name, "bob");
            assert_eq!(row.data.rank, 10);
            Ok(())
        })
        .await?;

        Ok(())
    })
}

#[test]
fn fetch_one_by_id_should_error_with_row_not_found_for_missing_id() -> Result<(), C3p0Error> {
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub struct TestData {
        pub name: String,
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

        pool.transaction::<_, C3p0Error, _>(async |conn| {
            conn.create_table_if_not_exists::<TestData>().await?;
            conn.delete_all::<TestData>().await?;
            Ok(())
        })
        .await?;

        pool.transaction::<_, C3p0Error, _>(async |conn| {
            // Pick an id that cannot exist in a freshly cleared table.
            let result = conn.fetch_one_by_id::<TestData>(99_999_999).await;
            match result {
                Ok(_) => panic!("expected an error for a missing id"),
                Err(C3p0Error::SqlxError(sqlx::Error::RowNotFound)) => {}
                Err(other) => panic!(
                    "expected SqlxError(RowNotFound), got {other:?}; \
                     in particular this must NOT be an OptimisticLockError"
                ),
            }
            Ok(())
        })
        .await
    })
}

#[test]
fn update_should_fail_with_optimistic_lock_after_concurrent_commit() -> Result<(), C3p0Error> {
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub struct TestData {
        pub name: String,
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

        // Seed one record and capture its initial state.
        let original: Record<TestData> = pool
            .transaction::<_, C3p0Error, _>(async |conn| {
                conn.create_table_if_not_exists::<TestData>().await?;
                conn.delete_all::<TestData>().await?;
                let saved = conn
                    .save(NewRecord::new(TestData {
                        name: "v0".to_owned(),
                    }))
                    .await?;
                Ok(saved)
            })
            .await?;

        // tx-B: separate transaction commits a successful update, bumping version.
        pool.transaction::<_, C3p0Error, _>(async |conn| {
            let mut to_update = original.clone();
            to_update.data.name = "v1_from_tx_b".to_owned();
            let updated = conn.update(to_update).await?;
            assert_eq!(updated.version, original.version + 1);
            Ok(())
        })
        .await?;

        // tx-A: tries to commit its own update using the *stale* `original` it read
        // before tx-B committed. The version bound in the WHERE clause no longer
        // matches the row, so this must fail with `OptimisticLockError` — that's
        // the whole point of the version column.
        let result = pool
            .transaction::<_, C3p0Error, _>(async |conn| {
                let mut to_update = original.clone();
                to_update.data.name = "v1_from_tx_a".to_owned();
                conn.update(to_update).await?;
                Ok(())
            })
            .await;

        match result {
            Ok(()) => panic!("tx-A must not have been allowed to overwrite tx-B's commit"),
            Err(C3p0Error::OptimisticLockError { cause }) => {
                assert!(cause.contains(<TestData as c3p0::DataType>::TABLE_NAME));
                assert!(cause.contains(&format!(
                    "id [{:?}], version [{}]",
                    original.id, original.version
                )));
            }
            Err(other) => panic!("expected OptimisticLockError, got {other:?}"),
        }

        // Sanity check: the value persisted in the DB is tx-B's, not tx-A's.
        let final_state: Record<TestData> = pool
            .transaction::<_, C3p0Error, _>(async |conn| {
                conn.fetch_one_by_id::<TestData>(original.id).await
            })
            .await?;
        assert_eq!(final_state.data.name, "v1_from_tx_b");
        assert_eq!(final_state.version, original.version + 1);

        Ok(())
    })
}

#[test]
fn drop_table_with_cascade_should_drop_dependent_objects_on_postgres() -> Result<(), C3p0Error> {
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub struct TestData {
        pub name: String,
    }

    impl c3p0::DataType for TestData {
        const TABLE_NAME: &'static str =
            const_format::concatcp!("TEST_TABLE_", const_random::const_random!(u64));
        type CODEC = Self;
    }

    run_test(async {
        // Postgres is the only backend whose `drop_table_if_exists(cascade)` actually
        // emits `DROP TABLE … CASCADE`. MySQL parses the keyword for compatibility
        // but does not propagate the drop; SQLite ignores the flag entirely. The
        // observable behaviour we want to test (cascade tearing down a dependent
        // view) only exists on Postgres, so the test is a no-op elsewhere.
        if db_specific::db_type() != DbType::Pg {
            return Ok(());
        }

        let data = data(false).await;
        let pool = &data.0;

        pool.transaction::<_, C3p0Error, _>(async |conn| {
            conn.drop_table_if_exists::<TestData>(true).await?;
            conn.create_table_if_not_exists::<TestData>().await?;
            Ok(())
        })
        .await?;

        // Create a view that depends on the c3p0-managed table. This forces the
        // backend to refuse a non-cascading drop and to succeed on a cascading one.
        let table = <TestData as c3p0::DataType>::TABLE_NAME;
        let view = format!("{table}_dep_view");
        let create_view_sql = format!("CREATE OR REPLACE VIEW {view} AS SELECT id FROM {table}");
        sqlx::query(sqlx::AssertSqlSafe(create_view_sql))
            .execute(pool.pool())
            .await
            .map_err(C3p0Error::from)?;

        // Drop without cascade must fail because of the dependent view.
        let no_cascade_result = pool
            .transaction::<_, C3p0Error, _>(async |conn| {
                conn.drop_table_if_exists::<TestData>(false).await?;
                Ok(())
            })
            .await;
        assert!(
            no_cascade_result.is_err(),
            "DROP TABLE without CASCADE must fail when a dependent view exists; got Ok"
        );

        // Drop *with* cascade must succeed AND must take the dependent view with it.
        pool.transaction::<_, C3p0Error, _>(async |conn| {
            conn.drop_table_if_exists::<TestData>(true).await?;
            Ok(())
        })
        .await?;

        // Verify: the view is gone. If CASCADE didn't propagate, this query would
        // succeed (or fail with "relation … does not exist for the table" only),
        // so we explicitly assert the view was dropped.
        let view_exists: bool = sqlx::query_scalar(
            "SELECT EXISTS (SELECT 1 FROM pg_catalog.pg_views WHERE viewname = $1)",
        )
        .bind(&view)
        .fetch_one(pool.pool())
        .await
        .map_err(C3p0Error::from)?;
        assert!(
            !view_exists,
            "CASCADE should have dropped the dependent view {view}, but it still exists"
        );

        Ok(())
    })
}
