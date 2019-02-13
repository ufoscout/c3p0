#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

use diesel::pg::PgConnection;
use diesel::prelude::*;

use testcontainers::*;

use serde_json::Value;

embed_migrations!("./migrations/");

pub fn establish_connection() -> PgConnection {

    let docker = clients::Cli::default();
    let node = docker.run(postgres_image());

    let database_url = format!(
        "postgres://postgres:postgres@127.0.0.1:{}/postgres",
        node.get_host_port(5432).unwrap()
    );

    PgConnection::establish(&database_url).expect(&format!("Error connecting to {}", database_url))
}

pub fn upgrade_db(conn: &PgConnection) {
    // This will run the necessary migrations.
    //embedded_migrations::run(connection);

    // By default the output is thrown out. If you want to redirect it to stdout, you
    // should call embedded_migrations::run_with_output.
    embedded_migrations::run_with_output(conn, &mut std::io::stdout())
        .expect(&format!("Should run the migrations"));
}
/*
pub fn create_post<'a>(conn: &PgConnection, title: &'a str, body: &'a str) -> Post {
    use schema::posts;

    let new_post = NewPost {
        title: title,
        body: body,
    };

    diesel::insert_into(posts::table)
        .values(&new_post)
        .get_result(conn)
        .expect("Error saving new post")
}
*/

#[test]
fn should_perform_a_query() {
    let conn = establish_connection();
    upgrade_db(&conn);

    let new_data = models::NewTestData {
        version: 0,
        data: models::CustomValue{name: "hello".to_owned()}
    };

    let saved_data: models::TestData = diesel::insert_into(schema::test_table::table)
        .values(&new_data)
        .get_result(&conn)
        .expect("Error saving new post");

    /*
    let new_post = create_post(&connection, "my_post_title", "my_post_body");
    println!("Created post with id {}", new_post.id);

    let post = diesel::update(posts::table.find(new_post.id))
        .set(posts::published.eq(true))
        .get_result::<Post>(&connection)
        .expect(&format!("Unable to find post {}", new_post.id));
    println!("Published post {}", post.title);

    let results = posts::table
        .filter(posts::published.eq(true))
        .limit(5)
        .load::<Post>(&connection)
        .expect("Error loading posts");

    println!("Displaying {} posts", results.len());
    for post in &results {
        println!("{}", post.title);
        println!("----------\n");
        println!("{}", post.body);
    }

    assert!(results.len() > 0);

    let num_deleted = diesel::delete(posts::table.filter(posts::id.eq(post.id)))
        .execute(&connection)
        .expect("Error deleting posts");

    assert_eq!(1, num_deleted);
    */
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
    use serde_json::Value;
    use serde_derive::{Deserialize, Serialize};

    #[derive(Insertable, C3p0Model)]
    #[table_name = "test_table"]
    pub struct NewTestData {
        pub version: i32,
        pub data: CustomValue,
    }

    #[derive(Queryable, C3p0Model)]
    pub struct TestData {
        pub id: i64,
        pub version: i32,
        pub data: CustomValue,
    }

    //use diesel::types::{Json, Jsonb};

    use c3p0_diesel_macro::*;

    #[derive(Serialize, Deserialize, Debug, DieselJson)]
    pub struct CustomValue {
        pub name: String
    }
}

fn postgres_image() -> testcontainers::images::generic::GenericImage {
    testcontainers::images::generic::GenericImage::new("postgres:11-alpine")
        .with_wait_for(images::generic::WaitFor::message_on_stderr(
            "database system is ready to accept connections",
        ))
}