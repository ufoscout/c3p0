use crate::shared::TestData;
use c3p0::{C3p0, C3p0Repository, NewModel};
use c3p0_pg_generic::PostgresManagerBuilder;

mod shared;

#[test]
fn should_create_and_drop_table() {
    shared::SINGLETON.get(|(pool, _)| {
        let mut conn = pool.get().unwrap();
        let conf = PostgresManagerBuilder::new("TEST_TABLE").build();

        let jpo = C3p0Repository::build(conf);

        let model = NewModel::new(TestData {
            first_name: "my_first_name".to_owned(),
            last_name: "my_last_name".to_owned(),
        });

        assert!(jpo.drop_table_if_exists(&mut conn).is_ok());
        assert!(jpo.save(&mut conn, model.clone()).is_err());

        println!("first {:?}", jpo.create_table_if_not_exists(&mut conn));

        assert!(jpo.create_table_if_not_exists(&mut conn).is_ok());
        assert!(jpo.create_table_if_not_exists(&mut conn).is_ok());
        assert!(jpo.save(&mut conn, model.clone()).is_ok());

        assert!(jpo.drop_table_if_exists(&mut conn).is_ok());
        assert!(jpo.drop_table_if_exists(&mut conn).is_ok());
        assert!(jpo.save(&mut conn, model.clone()).is_err());

        println!("second {:?}", jpo.create_table_if_not_exists(&mut conn));

        assert!(jpo.create_table_if_not_exists(&mut conn).is_ok());
        assert!(jpo.save(&mut conn, model.clone()).is_ok());
    });
}

#[test]
fn postgres_basic_crud() {
    shared::SINGLETON.get(|(pool, _)| {
        let mut conn = pool.get().unwrap();
        let conf = PostgresManagerBuilder::new("TEST_TABLE").build();

        let jpo = C3p0Repository::build(conf);

        assert!(jpo.create_table_if_not_exists(&mut conn).is_ok());
        jpo.delete_all(&mut conn).unwrap();

        let model = NewModel::new(TestData {
            first_name: "my_first_name".to_owned(),
            last_name: "my_last_name".to_owned(),
        });

        let saved_model = jpo.save(&mut conn, model.clone()).unwrap();
        assert!(saved_model.id >= 0);

        let found_model = jpo.find_by_id(&mut conn, &saved_model).unwrap().unwrap();
        assert_eq!(saved_model.id, found_model.id);
        assert_eq!(saved_model.version, found_model.version);
        assert_eq!(saved_model.data.first_name, found_model.data.first_name);
        assert_eq!(saved_model.data.last_name, found_model.data.last_name);

        let deleted = jpo.delete_by_id(&mut conn, &saved_model).unwrap();
        assert_eq!(1, deleted);
    });
}

#[test]
fn should_find_all() {
    shared::SINGLETON.get(|(pool, _)| {
        let mut conn = pool.get().unwrap();
        let conf = PostgresManagerBuilder::new("TEST_TABLE")
            .with_schema_name("public")
            .build();
        let jpo = C3p0Repository::build(conf);

        assert!(jpo.create_table_if_not_exists(&mut conn).is_ok());
        jpo.delete_all(&mut conn).unwrap();

        let model = NewModel::new(TestData {
            first_name: "my_first_name".to_owned(),
            last_name: "my_last_name".to_owned(),
        });

        let saved_model_0 = jpo.save(&mut conn, model.clone()).unwrap();
        let saved_model_1 = jpo.save(&mut conn, model.clone()).unwrap();
        let saved_model_2 = jpo.save(&mut conn, model.clone()).unwrap();

        let models = jpo.find_all(&mut conn).unwrap();

        assert_eq!(3, models.len());
        assert_eq!(saved_model_0.id, models[0].id);
        assert_eq!(saved_model_1.id, models[1].id);
        assert_eq!(saved_model_2.id, models[2].id);
    });
}

#[test]
fn should_delete_all() {
    shared::SINGLETON.get(|(pool, _)| {
        let mut conn = pool.get().unwrap();
        let conf = PostgresManagerBuilder::new("TEST_TABLE").build();
        let jpo = C3p0Repository::build(conf);
        assert!(jpo.drop_table_if_exists(&mut conn).is_ok());
        assert!(jpo.create_table_if_not_exists(&mut conn).is_ok());

        let model = NewModel::new(TestData {
            first_name: "my_first_name".to_owned(),
            last_name: "my_last_name".to_owned(),
        });

        let model1 = jpo.save(&mut conn, model.clone()).unwrap();
        jpo.save(&mut conn, model.clone()).unwrap();
        jpo.save(&mut conn, model.clone()).unwrap();

        assert!(jpo.find_by_id(&mut conn, &model1.id).unwrap().is_some());
        assert_eq!(1, jpo.delete_by_id(&mut conn, &model1.id).unwrap());
        assert!(jpo.find_by_id(&mut conn, &model1).unwrap().is_none());
        assert_eq!(2, jpo.count_all(&mut conn).unwrap());

        assert_eq!(2, jpo.delete_all(&mut conn).unwrap());
        assert_eq!(0, jpo.count_all(&mut conn).unwrap());
    });
}

#[test]
fn should_count() {
    shared::SINGLETON.get(|(pool, _)| {
        let mut conn = pool.get().unwrap();
        let conf = PostgresManagerBuilder::new("TEST_TABLE").build();
        let jpo = C3p0Repository::build(conf);

        assert!(jpo.create_table_if_not_exists(&mut conn).is_ok());
        assert!(jpo.delete_all(&mut conn).is_ok());

        assert_eq!(0, jpo.count_all(&mut conn).unwrap());

        let model = NewModel::new(TestData {
            first_name: "my_first_name".to_owned(),
            last_name: "my_last_name".to_owned(),
        });

        jpo.save(&mut conn, model.clone()).unwrap();
        assert_eq!(1, jpo.count_all(&mut conn).unwrap());

        jpo.save(&mut conn, model.clone()).unwrap();
        assert_eq!(2, jpo.count_all(&mut conn).unwrap());

        jpo.save(&mut conn, model.clone()).unwrap();
        assert_eq!(3, jpo.count_all(&mut conn).unwrap());

        assert_eq!(3, jpo.delete_all(&mut conn).unwrap());
        assert_eq!(0, jpo.count_all(&mut conn).unwrap());
    });
}

#[test]
fn should_return_whether_exists_by_id() {
    shared::SINGLETON.get(|(pool, _)| {
        let mut conn = pool.get().unwrap();
        let conf = PostgresManagerBuilder::new("TEST_TABLE").build();
        let jpo = C3p0Repository::build(conf);

        assert!(jpo.create_table_if_not_exists(&mut conn).is_ok());

        let model = NewModel::new(TestData {
            first_name: "my_first_name".to_owned(),
            last_name: "my_last_name".to_owned(),
        });

        let model = jpo.save(&mut conn, model.clone()).unwrap();
        assert!(jpo.exists_by_id(&mut conn, &model).unwrap());
        assert!(jpo.exists_by_id(&mut conn, &model.id).unwrap());

        assert_eq!(1, jpo.delete_by_id(&mut conn, &model).unwrap());
        assert!(!jpo.exists_by_id(&mut conn, &model).unwrap());
        assert!(!jpo.exists_by_id(&mut conn, &model.id).unwrap());
    });
}
