use crate::tests::util::rand_string;
use crate::*;

#[test]
fn should_create_and_drop_table() {
    SINGLETON.get(|(pool, _)| {
        let conn = pool.connection().unwrap();
        let table_name = format!("TEST_TABLE_{}", rand_string(8));
        let jpo = C3p0JsonBuilder::new(table_name).build();

        let model = NewModel::new(TestData {
            first_name: "my_first_name".to_owned(),
            last_name: "my_last_name".to_owned(),
        });

        assert!(jpo.drop_table_if_exists(&conn).is_ok());
        assert!(jpo.save(&conn, model.clone()).is_err());

        println!("first {:?}", jpo.create_table_if_not_exists(&conn));

        assert!(jpo.create_table_if_not_exists(&conn).is_ok());
        assert!(jpo.create_table_if_not_exists(&conn).is_ok());

        assert!(jpo.save(&conn, model.clone()).is_ok());

        assert!(jpo.drop_table_if_exists(&conn).is_ok());
        assert!(jpo.drop_table_if_exists(&conn).is_ok());
        assert!(jpo.save(&conn, model.clone()).is_err());

        println!("second {:?}", jpo.create_table_if_not_exists(&conn));

        assert!(jpo.create_table_if_not_exists(&conn).is_ok());
        assert!(jpo.save(&conn, model.clone()).is_ok());
    });
}

#[test]
fn basic_crud() {
    SINGLETON.get(|(pool, _)| {
        let conn = pool.connection().unwrap();
        let table_name = format!("TEST_TABLE_{}", rand_string(8));
        let jpo = C3p0JsonBuilder::new(table_name).build();

        assert!(jpo.create_table_if_not_exists(&conn).is_ok());
        jpo.delete_all(&conn).unwrap();

        let model = NewModel::new(TestData {
            first_name: "my_first_name".to_owned(),
            last_name: "my_last_name".to_owned(),
        });

        let saved_model = jpo.save(&conn, model.clone()).unwrap();
        assert!(saved_model.id >= 0);

        let found_model = jpo.fetch_one_by_id(&conn, &saved_model).unwrap().unwrap();
        assert_eq!(saved_model.id, found_model.id);
        assert_eq!(saved_model.version, found_model.version);
        assert_eq!(saved_model.data.first_name, found_model.data.first_name);
        assert_eq!(saved_model.data.last_name, found_model.data.last_name);

        let deleted = jpo.delete_by_id(&conn, &saved_model).unwrap();
        assert_eq!(1, deleted);
    });
}

#[test]
fn should_fetch_all() {
    SINGLETON.get(|(pool, _)| {
        let conn = pool.connection().unwrap();
        let table_name = format!("TEST_TABLE_{}", rand_string(8));
        let jpo = C3p0JsonBuilder::new(table_name).build();

        assert!(jpo.create_table_if_not_exists(&conn).is_ok());
        jpo.delete_all(&conn).unwrap();

        let model = NewModel::new(TestData {
            first_name: "my_first_name".to_owned(),
            last_name: "my_last_name".to_owned(),
        });

        let saved_model_0 = jpo.save(&conn, model.clone()).unwrap();
        let saved_model_1 = jpo.save(&conn, model.clone()).unwrap();
        let saved_model_2 = jpo.save(&conn, model.clone()).unwrap();

        let models = jpo.fetch_all(&conn).unwrap();

        assert_eq!(3, models.len());
        assert_eq!(saved_model_0.id, models[0].id);
        assert_eq!(saved_model_1.id, models[1].id);
        assert_eq!(saved_model_2.id, models[2].id);
    });
}

#[test]
fn should_delete_all() {
    SINGLETON.get(|(pool, _)| {
        let conn = pool.connection().unwrap();
        let table_name = format!("TEST_TABLE_{}", rand_string(8));
        let jpo = C3p0JsonBuilder::new(table_name).build();

        assert!(jpo.drop_table_if_exists(&conn).is_ok());
        assert!(jpo.create_table_if_not_exists(&conn).is_ok());

        let model = NewModel::new(TestData {
            first_name: "my_first_name".to_owned(),
            last_name: "my_last_name".to_owned(),
        });

        let model1 = jpo.save(&conn, model.clone()).unwrap();
        jpo.save(&conn, model.clone()).unwrap();
        jpo.save(&conn, model.clone()).unwrap();

        assert!(jpo.fetch_one_by_id(&conn, &model1.id).unwrap().is_some());
        assert_eq!(1, jpo.delete_by_id(&conn, &model1.id).unwrap());
        assert!(jpo.fetch_one_by_id(&conn, &model1).unwrap().is_none());
        assert_eq!(2, jpo.count_all(&conn).unwrap());

        assert_eq!(2, jpo.delete_all(&conn).unwrap());
        assert_eq!(0, jpo.count_all(&conn).unwrap());
    });
}

#[test]
fn should_count() {
    SINGLETON.get(|(pool, _)| {
        let conn = pool.connection().unwrap();
        let table_name = format!("TEST_TABLE_{}", rand_string(8));
        let jpo = C3p0JsonBuilder::new(table_name).build();

        assert!(jpo.create_table_if_not_exists(&conn).is_ok());
        assert!(jpo.delete_all(&conn).is_ok());

        assert_eq!(0, jpo.count_all(&conn).unwrap());

        let model = NewModel::new(TestData {
            first_name: "my_first_name".to_owned(),
            last_name: "my_last_name".to_owned(),
        });

        jpo.save(&conn, model.clone()).unwrap();
        assert_eq!(1, jpo.count_all(&conn).unwrap());

        jpo.save(&conn, model.clone()).unwrap();
        assert_eq!(2, jpo.count_all(&conn).unwrap());

        jpo.save(&conn, model.clone()).unwrap();
        assert_eq!(3, jpo.count_all(&conn).unwrap());

        assert_eq!(3, jpo.delete_all(&conn).unwrap());
        assert_eq!(0, jpo.count_all(&conn).unwrap());
    });
}

#[test]
fn should_return_whether_exists_by_id() {
    SINGLETON.get(|(pool, _)| {
        let conn = pool.connection().unwrap();
        let table_name = format!("TEST_TABLE_{}", rand_string(8));
        let jpo = C3p0JsonBuilder::new(table_name).build();

        assert!(jpo.create_table_if_not_exists(&conn).is_ok());

        let model = NewModel::new(TestData {
            first_name: "my_first_name".to_owned(),
            last_name: "my_last_name".to_owned(),
        });

        let model = jpo.save(&conn, model.clone()).unwrap();
        assert!(jpo.exists_by_id(&conn, &model).unwrap());
        assert!(jpo.exists_by_id(&conn, &model.id).unwrap());

        assert_eq!(1, jpo.delete_by_id(&conn, &model).unwrap());
        assert!(!jpo.exists_by_id(&conn, &model).unwrap());
        assert!(!jpo.exists_by_id(&conn, &model.id).unwrap());
    });
}

#[test]
fn should_update_and_increase_version() {
    SINGLETON.get(|(pool, _)| {
        let conn = pool.connection().unwrap();
        let table_name = format!("TEST_TABLE_{}", rand_string(8));
        let jpo = C3p0JsonBuilder::new(table_name).build();

        assert!(jpo.create_table_if_not_exists(&conn).is_ok());
        jpo.delete_all(&conn).unwrap();

        let model = NewModel::new(TestData {
            first_name: "my_first_name".to_owned(),
            last_name: "my_last_name".to_owned(),
        });

        let mut saved_model = jpo.save(&conn, model.clone()).unwrap();

        saved_model.data.first_name = "second_first_name".to_owned();
        let mut updated_model = jpo.update(&conn, saved_model.clone()).unwrap();
        assert_eq!(saved_model.id, updated_model.id);
        assert_eq!(saved_model.version + 1, updated_model.version);
        assert_eq!("second_first_name", updated_model.data.first_name);
        assert_eq!("my_last_name", updated_model.data.last_name);

        updated_model.data.last_name = "second_last_name".to_owned();
        updated_model = jpo.update(&conn, updated_model.clone()).unwrap();
        assert_eq!(saved_model.id, updated_model.id);
        assert_eq!(saved_model.version + 2, updated_model.version);
        assert_eq!("second_first_name", updated_model.data.first_name);
        assert_eq!("second_last_name", updated_model.data.last_name);

        let found_model = jpo.fetch_one_by_id(&conn, &saved_model).unwrap().unwrap();
        assert_eq!(found_model.id, updated_model.id);
        assert_eq!(found_model.version, updated_model.version);
        assert_eq!(found_model.data, updated_model.data);
    });
}

#[test]
fn update_should_return_optimistic_lock_exception() {
    SINGLETON.get(|(pool, _)| {
        let conn = pool.connection().unwrap();
        let table_name = format!("TEST_TABLE_{}", rand_string(8));
        let jpo = C3p0JsonBuilder::new(&table_name).build();

        assert!(jpo.create_table_if_not_exists(&conn).is_ok());
        jpo.delete_all(&conn).unwrap();

        let model = NewModel::new(TestData {
            first_name: "my_first_name".to_owned(),
            last_name: "my_last_name".to_owned(),
        });

        let mut saved_model = jpo.save(&conn, model.clone()).unwrap();

        saved_model.data.first_name = "second_first_name".to_owned();
        assert!(jpo.update(&conn, saved_model.clone()).is_ok());

        let expected_error = jpo.update(&conn, saved_model.clone());
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
    });
}

#[test]
fn should_delete_based_on_id_and_version() {
    SINGLETON.get(|(pool, _)| {
        let conn = pool.connection().unwrap();
        let table_name = format!("TEST_TABLE_{}", rand_string(8));
        let jpo = C3p0JsonBuilder::new(table_name).build();

        assert!(jpo.create_table_if_not_exists(&conn).is_ok());
        jpo.delete_all(&conn).unwrap();

        let model = NewModel::new(TestData {
            first_name: "my_first_name".to_owned(),
            last_name: "my_last_name".to_owned(),
        });

        let saved_model = jpo.save(&conn, model.clone()).unwrap();

        let deleted = jpo.delete(&conn, &saved_model).unwrap();
        assert_eq!(1, deleted);
        assert!(!jpo.exists_by_id(&conn, &saved_model).unwrap());
    });
}

#[test]
fn delete_should_return_optimistic_lock_exception() {
    SINGLETON.get(|(pool, _)| {
        let conn = pool.connection().unwrap();
        let table_name = format!("TEST_TABLE_{}", rand_string(8));
        let jpo = C3p0JsonBuilder::new(&table_name).build();

        assert!(jpo.create_table_if_not_exists(&conn).is_ok());
        jpo.delete_all(&conn).unwrap();

        let model = NewModel::new(TestData {
            first_name: "my_first_name".to_owned(),
            last_name: "my_last_name".to_owned(),
        });

        let saved_model = jpo.save(&conn, model.clone()).unwrap();
        assert!(jpo.update(&conn, saved_model.clone()).is_ok());

        let expected_error = jpo.delete(&conn, &saved_model);
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

        assert!(jpo.exists_by_id(&conn, &saved_model).unwrap());
    });

    #[test]
    fn should_fetch_by_sql() {
        SINGLETON.get(|(pool, _)| {
            let conn = pool.connection().unwrap();
            let table_name = format!("TEST_TABLE_{}", rand_string(8));
            let jpo = C3p0JsonBuilder::new(table_name.clone()).build();

            assert!(jpo.create_table_if_not_exists(&conn).is_ok());

            let model = NewModel::new(TestData {
                first_name: "my_first_name".to_owned(),
                last_name: "my_last_name".to_owned(),
            });

            let model = jpo.save(&conn, model.clone()).unwrap();

            let one = jpo
                .fetch_one_with_sql(
                    &conn,
                    &format!("select id, version, data from {}", table_name),
                    &[],
                )
                .unwrap();
            assert!(one.is_some());

            let all = jpo
                .fetch_all_with_sql(
                    &conn,
                    &format!("select id, version, data from {}", table_name),
                    &[],
                )
                .unwrap();
            assert!(!all.is_empty());
        });
    }
}
