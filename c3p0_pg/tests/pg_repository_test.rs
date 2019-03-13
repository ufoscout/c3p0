use crate::shared::TestData;
use c3p0_pg::{ConfigBuilder, JpoPg, NewModel, SimpleRepository};

mod shared;

#[test]
fn should_create_and_drop_table() {
    let _lock = shared::LOCK.lock().unwrap();
    let conn = shared::new_connection();

    let conf = ConfigBuilder::new("TEST_TABLE").build();

    let jpo = SimpleRepository::build(conf);

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
}

#[test]
fn postgres_basic_crud() {
    let _lock = shared::LOCK.lock().unwrap();
    let conn = shared::new_connection();

    let conf = ConfigBuilder::new("TEST_TABLE").build();

    let jpo = SimpleRepository::build(conf);

    assert!(jpo.create_table_if_not_exists(&conn).is_ok());
    jpo.delete_all(&conn).unwrap();

    let model = NewModel::new(TestData {
        first_name: "my_first_name".to_owned(),
        last_name: "my_last_name".to_owned(),
    });

    let saved_model = jpo.save(&conn, model.clone()).unwrap();
    assert!(saved_model.id >= 0);

    let found_model = jpo
        .find_by_id(&conn, saved_model.id)
        .unwrap()
        .unwrap();
    assert_eq!(saved_model.id, found_model.id);
    assert_eq!(saved_model.version, found_model.version);
    assert_eq!(saved_model.data.first_name, found_model.data.first_name);
    assert_eq!(saved_model.data.last_name, found_model.data.last_name);

    let deleted = jpo.delete_by_id(&conn, saved_model.id).unwrap();
    assert_eq!(1, deleted);
}

#[test]
fn should_find_all() {
    let _lock = shared::LOCK.lock().unwrap();

    let conn = shared::new_connection();

    let conf = ConfigBuilder::new("TEST_TABLE")
        .with_schema_name("public".to_owned())
        .build();
    let jpo = SimpleRepository::build(conf);

    assert!(jpo.create_table_if_not_exists(&conn).is_ok());
    jpo.delete_all(&conn).unwrap();

    let model = NewModel::new(TestData {
        first_name: "my_first_name".to_owned(),
        last_name: "my_last_name".to_owned(),
    });

    let saved_model_0 = jpo.save(&conn, model.clone()).unwrap();
    let saved_model_1 = jpo.save(&conn, model.clone()).unwrap();
    let saved_model_2 = jpo.save(&conn, model.clone()).unwrap();

    let models = jpo.find_all(&conn).unwrap();

    assert_eq!(3, models.len());
    assert_eq!(saved_model_0.id, models[0].id);
    assert_eq!(saved_model_1.id, models[1].id);
    assert_eq!(saved_model_2.id, models[2].id);
}

#[test]
fn should_delete_all() {
    let _lock = shared::LOCK.lock().unwrap();
    let conn = shared::new_connection();
    let conf = ConfigBuilder::new("TEST_TABLE").build();
    let jpo = SimpleRepository::build(conf);
    assert!(jpo.drop_table_if_exists(&conn).is_ok());
    assert!(jpo.create_table_if_not_exists(&conn).is_ok());

    let model = NewModel::new(TestData {
        first_name: "my_first_name".to_owned(),
        last_name: "my_last_name".to_owned(),
    });

    let model1 = jpo.save(&conn, model.clone()).unwrap();
    jpo.save(&conn, model.clone()).unwrap();
    jpo.save(&conn, model.clone()).unwrap();

    assert!(jpo.find_by_id(&conn, model1.id).unwrap().is_some());
    assert_eq!(1, jpo.delete_by_id(&conn, model1.id).unwrap());
    assert!(jpo.find_by_id(&conn, model1.id).unwrap().is_none());
    assert_eq!(2, jpo.find_all(&conn).unwrap().len());

    assert_eq!(2, jpo.delete_all(&conn).unwrap());
    assert_eq!(0, jpo.find_all(&conn).unwrap().len());
}
