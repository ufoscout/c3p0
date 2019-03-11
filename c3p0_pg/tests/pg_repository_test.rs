use c3p0_pg::{ConfigBuilder, JpoPg, Model, SimpleRepository};

mod shared;

#[test]
fn postgres_basic_crud() {
    let conn = shared::new_connection();

    let conf = ConfigBuilder::new("TEST_TABLE").build();

    let jpo = SimpleRepository::build(conf);

    assert_eq!(0, jpo.create_table_if_not_exists(&conn).unwrap());
    // assert_eq!(0, jpo.create_table_if_not_exists(&conn).unwrap());

    let model = Model::new(shared::TestData {
        first_name: "my_first_name".to_owned(),
        last_name: "my_last_name".to_owned(),
    });

    let saved_model = jpo.save(&conn, model.clone());
    assert!(saved_model.id.is_some());

    assert!(model.id.is_none());

    let found_model = jpo.find_by_id(&conn, saved_model.id.unwrap()).unwrap();
    assert_eq!(saved_model.id, found_model.id);
    assert_eq!(saved_model.version, found_model.version);
    assert_eq!(saved_model.data.first_name, found_model.data.first_name);
    assert_eq!(saved_model.data.last_name, found_model.data.last_name);
}
