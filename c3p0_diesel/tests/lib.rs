#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

use diesel::pg::PgConnection;
use diesel::prelude::*;

use testcontainers::*;

use c3p0_diesel::{JpoDiesel, SimpleRepository};
use serde_json::Value;
use crate::models::CustomValue;

embed_migrations!("./migrations/");

pub fn establish_connection() -> PgConnection {
    let docker = clients::Cli::default();
    let node = docker.run(postgres_image());

    let database_url = format!(
        "postgres://postgres:postgres@127.0.0.1:{}/postgres",
        node.get_host_port(5432).unwrap()
    );

    let conn = PgConnection::establish(&database_url).expect(&format!("Error connecting to {}", database_url));
    // This will run the necessary migrations.
    //embedded_migrations::run(connection);

    // By default the output is thrown out. If you want to redirect it to stdout, you
    // should call embedded_migrations::run_with_output.
    embedded_migrations::run_with_output(&conn, &mut std::io::stdout())
        .expect(&format!("Should run the migrations"));

    conn
}


#[test]
fn should_perform_a_query_with_diesel_json() {

    use schema::test_table::dsl as tt_dsl;

    let conn = establish_connection();

    let new_data = models::NewTestData {
        version: 0,
        data: models::CustomValue {
            name: "hello".to_owned(),
        },
    };

    let saved_data: models::TestData = diesel::insert_into(tt_dsl::test_table)
            .values(&new_data)
            .get_result(&conn)
            .expect("Error saving new TestData");


    println!("Created data with id {}", saved_data.id);

    let updated_data = diesel::update(tt_dsl::test_table)
            .set(tt_dsl::data.eq(CustomValue{name: "custom new value".to_owned()}))
            .get_result::<models::TestData>(&conn)
            .expect(&format!("Unable to find TestData id {}", saved_data.id));
    println!("Updated data [{}]", updated_data.data.name);

    let found_by_id = tt_dsl::test_table
        .filter(tt_dsl::id.eq(saved_data.id))
        .load::<models::TestData>(&conn)
        .expect("Error loading data");

    let results = tt_dsl::test_table
        //.filter(schema::test_table::published.eq(true))
        .limit(5)
        .load::<models::TestData>(&conn)
        .expect("Error loading data");

    println!("Displaying {} data", results.len());
    for data in &results {
        println!("{}", data.id);
        println!("----------\n");
        println!("{}", data.version);
    }

    assert!(results.len() > 0);

    let num_deleted =
        diesel::delete(tt_dsl::test_table.filter(tt_dsl::id.eq(saved_data.id)))
            .execute(&conn)
            .expect("Error deleting data");

    assert_eq!(1, num_deleted);
}

mod schema {

    table! {
        test_table (id) {
            id -> Int8,
            version -> Int4,
            data -> Jsonb,
        }
    }

}

mod models {
    use super::schema::*;
    use serde_derive::*;
    use serde_json::Value;

    #[derive(Insertable)]
    #[table_name = "test_table"]
    pub struct NewTestData {
        pub version: i32,
        pub data: CustomValue,
    }

    #[derive(Queryable)]
    pub struct TestData {
        pub id: i64,
        pub version: i32,
        pub data: CustomValue,
    }

    //use diesel::types::{Json, Jsonb};

    use c3p0_diesel_macro::*;

    #[derive(Serialize, Deserialize, Debug, DieselJson)]
    pub struct CustomValue {
        pub name: String,
    }
}

fn postgres_image() -> testcontainers::images::generic::GenericImage {
    testcontainers::images::generic::GenericImage::new("postgres:11-alpine").with_wait_for(
        images::generic::WaitFor::message_on_stderr(
            "database system is ready to accept connections",
        ),
    )
}
