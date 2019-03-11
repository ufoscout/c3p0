#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

use diesel::pg::PgConnection;
use diesel::prelude::*;

use testcontainers::*;

use crate::models::CustomValue;
use testcontainers::images::postgres::Postgres;

#[test]
fn should_perform_a_query_with_diesel_json() {
    use schema::test_table::dsl as tt_dsl;

    let docker = clients::Cli::default();
    let postgres_node = establish_connection(&docker);
    let conn = postgres_node.0;

    let new_data = models::NewCustomValueModel {
        version: 0,
        data: models::CustomValue {
            name: "hello".to_owned(),
        },
    };

    let saved_data: models::CustomValueModel = diesel::insert_into(tt_dsl::test_table)
        .values(&new_data)
        .get_result(&conn)
        .expect("Error saving new TestData");

    println!("Created data with id {}", saved_data.id);

    let updated_data = diesel::update(tt_dsl::test_table)
        .set(tt_dsl::data.eq(CustomValue {
            name: "custom new value".to_owned(),
        }))
        .get_result::<models::CustomValueModel>(&conn)
        .expect(&format!("Unable to find TestData id {}", saved_data.id));
    println!("Updated data [{}]", updated_data.data.name);

    let results = tt_dsl::test_table
        //.filter(schema::test_table::published.eq(true))
        .limit(5)
        .load::<models::CustomValueModel>(&conn)
        .expect("Error loading data");

    println!("Displaying {} data", results.len());
    for data in &results {
        println!("{}", data.id);
        println!("----------\n");
        println!("{}", data.version);
    }

    assert!(results.len() > 0);

    let num_deleted = diesel::delete(tt_dsl::test_table.filter(tt_dsl::id.eq(saved_data.id)))
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

    use c3p0_diesel_macro::*;

    #[derive(Serialize, Deserialize, Debug, C3p0Model)]
    #[table_name = "test_table"]
    pub struct CustomValue {
        pub name: String,
    }
}

embed_migrations!("./migrations/");

pub fn establish_connection(docker: &clients::Cli) -> (PgConnection, Container<clients::Cli, Postgres>) {
    let node = docker.run(images::postgres::Postgres::default());

    let database_url = format!(
        "postgres://postgres:postgres@127.0.0.1:{}/postgres",
        node.get_host_port(5432).unwrap()
    );

    let conn = PgConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url));
    // This will run the necessary migrations.
    //embedded_migrations::run(connection);

    // By default the output is thrown out. If you want to redirect it to stdout, you
    // should call embedded_migrations::run_with_output.
    embedded_migrations::run_with_output(&conn, &mut std::io::stdout())
        .expect(&format!("Should run the migrations"));

    (conn, node)
}
