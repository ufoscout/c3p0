use c3p0_pg::{ConfigBuilder, JpoPg, Model, SimpleRepository};
use crate::shared::{TestModel, TestData};
use testcontainers::clients;

mod shared;

#[test]
fn should_create_and_drop_table() {
    let docker = clients::Cli::default();
    let postgres_node = shared::new_connection(&docker);
    let conn = postgres_node.0;

    let conf = ConfigBuilder::new("TEST_TABLE").build();

    let jpo = SimpleRepository::build(conf);

    let model = Model::new(shared::TestData {
        first_name: "my_first_name".to_owned(),
        last_name: "my_last_name".to_owned(),
    });

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
    let docker = clients::Cli::default();
    let postgres_node = shared::new_connection(&docker);
    let conn = postgres_node.0;

    let conf = ConfigBuilder::new("TEST_TABLE").build();

    let jpo = SimpleRepository::build(conf);

    assert!(jpo.create_table_if_not_exists(&conn).is_ok());

    let model = Model::new(shared::TestData {
        first_name: "my_first_name".to_owned(),
        last_name: "my_last_name".to_owned(),
    });

    let saved_model = jpo.save(&conn, model.clone()).unwrap();
    assert!(saved_model.id.is_some());

    assert!(model.id.is_none());

    let found_model = jpo.find_by_id(&conn, saved_model.id.unwrap()).unwrap().unwrap();
    assert_eq!(saved_model.id, found_model.id);
    assert_eq!(saved_model.version, found_model.version);
    assert_eq!(saved_model.data.first_name, found_model.data.first_name);
    assert_eq!(saved_model.data.last_name, found_model.data.last_name);
}
