use serde::{Deserialize, Serialize};
use surrealdb::engine::local::Mem;
use surrealdb::sql::Thing;
use surrealdb::Surreal;

// #[derive(Debug, Serialize)]
// struct Name<'a> {
//     first: &'a str,
//     last: &'a str,
// }

// #[derive(Debug, Serialize)]
// struct Person<'a> {
//     title: &'a str,
//     name: Name<'a>,
//     marketing: bool,
// }

#[derive(Debug, Serialize, Deserialize)]
struct Name {
    first: String,
    last: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Person {
    title: String,
    name: Name,
    marketing: bool,
}

#[derive(Debug, Serialize)]
struct Responsibility {
    marketing: bool,
}

#[derive(Debug, Deserialize)]
struct Record {
    #[allow(dead_code)]
    id: Thing,
}

#[tokio::test]
async fn test_spike() -> surrealdb::Result<()> {
    // Create database connection
    let db = Surreal::new::<Mem>(()).await?;

    // Select a specific namespace / database
    db.use_ns("test").use_db("test").await?;

    // Create a new person with a random id
    let created: Vec<Record> = db
        .create("person")
        .content(Person {
            title: "Founder & CEO".to_owned(),
            name: Name {
                first: "Tobie".to_owned(),
                last: "Morgan Hitchcock".to_owned(),
            },
            marketing: true,
        })
        .await?;
    dbg!(created);

    // Update a person record with a specific id
    // let updated: Option<Record> = db
    //     .update(("person", "jaime"))
    //     .merge(Responsibility { marketing: true })
    //     .await?;
    // dbg!(updated);

    // Select all people records
    let all_records: Vec<Record> = db.select("person").await?;
    dbg!(all_records);

    let mut all_persons = db.query("SELECT * FROM person").await?;
    let all_persons: Vec<Person> = all_persons.take(0)?;
    dbg!(all_persons);

    // Select all people records
    // let all_people: Vec<Person> = db.select("person").await?;
    // dbg!(all_people);

    // Perform a custom advanced query
    let groups = db
        .query("SELECT marketing, count() FROM type::table($table) GROUP BY marketing")
        .bind(("table", "person"))
        .await?;
    dbg!(groups);

    Ok(())
}
