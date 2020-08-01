use crate::utils::*;
use crate::*;

#[test]
#[cfg(not(feature = "in_memory_blocking"))]
fn should_create_and_drop_table() -> Result<(), Box<dyn std::error::Error>> {
    if db_specific::db_type() == DbType::InMemory {
        return Ok(());
    }

    let data = data(false);
    let pool = &data.0;

    let table_name = format!("TEST_TABLE_{}", rand_string(8));
    let jpo = C3p0JsonBuilder::new(table_name).build();

    let model = NewModel::new(TestData {
        first_name: "my_first_name".to_owned(),
        last_name: "my_last_name".to_owned(),
    });

    pool.transaction::<_, C3p0Error, _>(|conn| {
        assert!(jpo.drop_table_if_exists(conn, false).is_ok());
        Ok(())
    })?;

    pool.transaction::<_, C3p0Error, _>(|conn| {
        assert!(jpo.save(conn, model.clone()).is_err());
        Ok(())
    })?;

    pool.transaction::<_, C3p0Error, _>(|conn| {
        println!("first {:?}", jpo.create_table_if_not_exists(conn));

        assert!(jpo.create_table_if_not_exists(conn).is_ok());
        assert!(jpo.create_table_if_not_exists(conn).is_ok());

        assert!(jpo.save(conn, model.clone()).is_ok());

        assert!(jpo.drop_table_if_exists(conn, false).is_ok());
        assert!(jpo.drop_table_if_exists(conn, true).is_ok());
        Ok(())
    })?;

    pool.transaction::<_, C3p0Error, _>(|conn| {
        assert!(jpo.save(conn, model.clone()).is_err());
        Ok(())
    })?;

    pool.transaction::<_, C3p0Error, _>(|conn| {
        println!("second {:?}", jpo.create_table_if_not_exists(conn));

        assert!(jpo.create_table_if_not_exists(conn).is_ok());
        assert!(jpo.save(conn, model.clone()).is_ok());
        Ok(())
    })?;

    Ok(())
}

#[test]
fn basic_crud() -> Result<(), Box<dyn std::error::Error>> {
    let data = data(false);
    let pool = &data.0;

    pool.transaction(|conn| {
        let table_name = format!("TEST_TABLE_{}", rand_string(8));
        let jpo = C3p0JsonBuilder::new(table_name).build();

        assert!(jpo.create_table_if_not_exists(conn).is_ok());
        jpo.delete_all(conn).unwrap();

        let model = NewModel::new(TestData {
            first_name: "my_first_name".to_owned(),
            last_name: "my_last_name".to_owned(),
        });

        let saved_model = jpo.save(conn, model.clone()).unwrap();
        assert!(saved_model.id >= 0);

        let found_model = jpo
            .fetch_one_optional_by_id(conn, &saved_model)
            .unwrap()
            .unwrap();
        assert_eq!(saved_model.id, found_model.id);
        assert_eq!(saved_model.version, found_model.version);
        assert_eq!(saved_model.data.first_name, found_model.data.first_name);
        assert_eq!(saved_model.data.last_name, found_model.data.last_name);

        let deleted = jpo.delete_by_id(conn, &saved_model).unwrap();
        assert_eq!(1, deleted);
        Ok(())
    })
}

#[test]
fn should_fetch_all() -> Result<(), Box<dyn std::error::Error>> {
    let data = data(false);
    let pool = &data.0;

    pool.transaction(|conn| {
        let table_name = format!("TEST_TABLE_{}", rand_string(8));
        let jpo = C3p0JsonBuilder::new(table_name).build();

        assert!(jpo.create_table_if_not_exists(conn).is_ok());
        jpo.delete_all(conn).unwrap();

        let model = NewModel::new(TestData {
            first_name: "my_first_name".to_owned(),
            last_name: "my_last_name".to_owned(),
        });

        let saved_model_0 = jpo.save(conn, model.clone()).unwrap();
        let saved_model_1 = jpo.save(conn, model.clone()).unwrap();
        let saved_model_2 = jpo.save(conn, model.clone()).unwrap();

        let models = jpo.fetch_all(conn).unwrap();

        assert_eq!(3, models.len());
        assert_eq!(saved_model_0.id, models[0].id);
        assert_eq!(saved_model_1.id, models[1].id);
        assert_eq!(saved_model_2.id, models[2].id);
        Ok(())
    })
}

#[test]
fn should_delete_all() -> Result<(), Box<dyn std::error::Error>> {
    let data = data(false);
    let pool = &data.0;

    pool.transaction(|conn| {
        let table_name = format!("TEST_TABLE_{}", rand_string(8));
        let jpo = C3p0JsonBuilder::new(table_name).build();

        assert!(jpo.drop_table_if_exists(conn, true).is_ok());
        assert!(jpo.create_table_if_not_exists(conn).is_ok());

        let model = NewModel::new(TestData {
            first_name: "my_first_name".to_owned(),
            last_name: "my_last_name".to_owned(),
        });

        let model1 = jpo.save(conn, model.clone()).unwrap();
        jpo.save(conn, model.clone()).unwrap();
        jpo.save(conn, model.clone()).unwrap();

        assert!(jpo.fetch_one_by_id(conn, &model1.id).is_ok());
        assert_eq!(1, jpo.delete_by_id(conn, &model1.id).unwrap());
        assert!(jpo.fetch_one_by_id(conn, &model1).is_err());
        assert_eq!(2, jpo.count_all(conn).unwrap());

        assert_eq!(2, jpo.delete_all(conn).unwrap());
        assert_eq!(0, jpo.count_all(conn).unwrap());
        Ok(())
    })
}

#[test]
fn should_count() -> Result<(), Box<dyn std::error::Error>> {
    let data = data(false);
    let pool = &data.0;

    pool.transaction(|conn| {
        let table_name = format!("TEST_TABLE_{}", rand_string(8));
        let jpo = C3p0JsonBuilder::new(table_name).build();

        assert!(jpo.create_table_if_not_exists(conn).is_ok());
        assert!(jpo.delete_all(conn).is_ok());

        assert_eq!(0, jpo.count_all(conn).unwrap());

        let model = NewModel::new(TestData {
            first_name: "my_first_name".to_owned(),
            last_name: "my_last_name".to_owned(),
        });

        jpo.save(conn, model.clone()).unwrap();
        assert_eq!(1, jpo.count_all(conn).unwrap());

        jpo.save(conn, model.clone()).unwrap();
        assert_eq!(2, jpo.count_all(conn).unwrap());

        jpo.save(conn, model.clone()).unwrap();
        assert_eq!(3, jpo.count_all(conn).unwrap());

        assert_eq!(3, jpo.delete_all(conn).unwrap());
        assert_eq!(0, jpo.count_all(conn).unwrap());

        Ok(())
    })
}

#[test]
fn should_return_whether_exists_by_id() -> Result<(), Box<dyn std::error::Error>> {
    let data = data(false);
    let pool = &data.0;

    pool.transaction(|conn| {
        let table_name = format!("TEST_TABLE_{}", rand_string(8));
        let jpo = C3p0JsonBuilder::new(table_name).build();

        assert!(jpo.create_table_if_not_exists(conn).is_ok());

        let model = NewModel::new(TestData {
            first_name: "my_first_name".to_owned(),
            last_name: "my_last_name".to_owned(),
        });

        let model = jpo.save(conn, model.clone()).unwrap();
        assert!(jpo.exists_by_id(conn, &model).unwrap());
        assert!(jpo.exists_by_id(conn, &model.id).unwrap());

        assert_eq!(1, jpo.delete_by_id(conn, &model).unwrap());
        assert!(!jpo.exists_by_id(conn, &model).unwrap());
        assert!(!jpo.exists_by_id(conn, &model.id).unwrap());
        Ok(())
    })
}

#[test]
fn should_update_and_increase_version() -> Result<(), Box<dyn std::error::Error>> {
    let data = data(false);
    let pool = &data.0;

    pool.transaction(|conn| {
        let table_name = format!("TEST_TABLE_{}", rand_string(8));
        let jpo = C3p0JsonBuilder::new(table_name).build();

        assert!(jpo.create_table_if_not_exists(conn).is_ok());
        jpo.delete_all(conn).unwrap();

        let model = NewModel::new(TestData {
            first_name: "my_first_name".to_owned(),
            last_name: "my_last_name".to_owned(),
        });

        let mut saved_model = jpo.save(conn, model.clone()).unwrap();

        saved_model.data.first_name = "second_first_name".to_owned();
        let mut updated_model = jpo.update(conn, saved_model.clone()).unwrap();
        assert_eq!(saved_model.id, updated_model.id);
        assert_eq!(saved_model.version + 1, updated_model.version);
        assert_eq!("second_first_name", updated_model.data.first_name);
        assert_eq!("my_last_name", updated_model.data.last_name);

        updated_model.data.last_name = "second_last_name".to_owned();
        updated_model = jpo.update(conn, updated_model.clone()).unwrap();
        assert_eq!(saved_model.id, updated_model.id);
        assert_eq!(saved_model.version + 2, updated_model.version);
        assert_eq!("second_first_name", updated_model.data.first_name);
        assert_eq!("second_last_name", updated_model.data.last_name);

        let found_model = jpo.fetch_one_by_id(conn, &saved_model).unwrap();
        assert_eq!(found_model.id, updated_model.id);
        assert_eq!(found_model.version, updated_model.version);
        assert_eq!(found_model.data, updated_model.data);
        Ok(())
    })
}

#[test]
fn update_should_return_optimistic_lock_exception() -> Result<(), Box<dyn std::error::Error>> {
    let data = data(false);
    let pool = &data.0;

    pool.transaction(|conn| {
        let table_name = format!("TEST_TABLE_{}", rand_string(8));
        let jpo = C3p0JsonBuilder::new(&table_name).build();

        assert!(jpo.create_table_if_not_exists(conn).is_ok());
        jpo.delete_all(conn).unwrap();

        let model = NewModel::new(TestData {
            first_name: "my_first_name".to_owned(),
            last_name: "my_last_name".to_owned(),
        });

        let mut saved_model = jpo.save(conn, model.clone()).unwrap();

        saved_model.data.first_name = "second_first_name".to_owned();
        assert!(jpo.update(conn, saved_model.clone()).is_ok());

        let expected_error = jpo.update(conn, saved_model.clone());
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
}

#[test]
fn should_delete_based_on_id_and_version() -> Result<(), Box<dyn std::error::Error>> {
    let data = data(false);
    let pool = &data.0;

    pool.transaction(|conn| {
        let table_name = format!("TEST_TABLE_{}", rand_string(8));
        let jpo = C3p0JsonBuilder::new(table_name).build();

        assert!(jpo.create_table_if_not_exists(conn).is_ok());
        jpo.delete_all(conn).unwrap();

        let model = NewModel::new(TestData {
            first_name: "my_first_name".to_owned(),
            last_name: "my_last_name".to_owned(),
        });

        let saved_model = jpo.save(conn, model.clone()).unwrap();

        let deleted = jpo.delete(conn, saved_model.clone()).unwrap();
        assert_eq!(saved_model.id, deleted.id);

        assert!(jpo.delete(conn, saved_model.clone()).is_err());

        assert!(!jpo.exists_by_id(conn, &saved_model).unwrap());

        Ok(())
    })
}

#[test]
fn delete_should_return_optimistic_lock_exception() -> Result<(), Box<dyn std::error::Error>> {
    let data = data(false);
    let pool = &data.0;

    pool.transaction(|conn| {
        let table_name = format!("TEST_TABLE_{}", rand_string(8));
        let jpo = C3p0JsonBuilder::new(&table_name).build();

        assert!(jpo.create_table_if_not_exists(conn).is_ok());
        jpo.delete_all(conn).unwrap();

        let model = NewModel::new(TestData {
            first_name: "my_first_name".to_owned(),
            last_name: "my_last_name".to_owned(),
        });

        let saved_model = jpo.save(conn, model.clone()).unwrap();
        assert!(jpo.update(conn, saved_model.clone()).is_ok());

        let expected_error = jpo.delete(conn, saved_model.clone());
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

        assert!(jpo.exists_by_id(conn, &saved_model).unwrap());

        Ok(())
    })
}

#[test]
fn json_should_perform_for_update_fetches() -> Result<(), Box<dyn std::error::Error>> {
    let data = data(false);
    let pool = &data.0;
    let c3p0: C3p0Impl = pool.clone();
    let table_name = format!("TEST_TABLE_{}", rand_string(8));
    let jpo = C3p0JsonBuilder::new(table_name).build();

    let model = NewModel::new(TestData {
        first_name: "my_first_name".to_owned(),
        last_name: "my_last_name".to_owned(),
    });

    let result: Result<_, C3p0Error> = c3p0.transaction(|conn| {
        assert!(jpo.create_table_if_not_exists(conn).is_ok());
        assert!(jpo.save(conn, model.clone()).is_ok());
        assert!(jpo.save(conn, model.clone()).is_ok());
        Ok(())
    });
    assert!(result.is_ok());

    // fetch all ForUpdate::Default
    let result: Result<_, C3p0Error> =
        c3p0.transaction(|conn| jpo.fetch_all_for_update(conn, &ForUpdate::Default));
    assert!(result.is_ok());

    if db_specific::db_type() != DbType::MySql && db_specific::db_type() != DbType::TiDB {
        // fetch one ForUpdate::NoWait
        let result: Result<_, C3p0Error> = c3p0.transaction(|conn| {
            jpo.fetch_one_optional_by_id_for_update(conn, &0, &ForUpdate::NoWait)
        });
        assert!(result.is_ok());

        // fetch one ForUpdate::SkipLocked
        let result: Result<_, C3p0Error> = c3p0.transaction(|conn| {
            jpo.fetch_one_optional_by_id_for_update(conn, &0, &ForUpdate::SkipLocked)
        });
        assert!(result.is_ok());
    }

    // fetch all ForUpdate::No
    let result: Result<_, C3p0Error> =
        c3p0.transaction(|conn| jpo.fetch_all_for_update(conn, &ForUpdate::No));
    assert!(result.is_ok());

    {
        assert!(pool
            .transaction(|conn| { jpo.drop_table_if_exists(conn, true) })
            .is_ok());
    }
    Ok(())
}
