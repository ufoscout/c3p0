[![crates.io](https://img.shields.io/crates/v/c3p0.svg)](https://crates.io/crates/c3p0)
![Build Status](https://github.com/ufoscout/c3p0/actions/workflows/build_and_test.yml/badge.svg)
[![codecov](https://codecov.io/gh/ufoscout/c3p0/branch/master/graph/badge.svg)](https://codecov.io/gh/ufoscout/c3p0)

![A pleasure to meet you. I am C-3p0, JSON-DB Relations.](https://raw.githubusercontent.com/ufoscout/c3p0/refs/heads/master/images/c3p0_1_small.png)

# "A pleasure to meet you. I am C-3p0, JSON-DB Relations."

__C3p0__: "Hello, I don't believe we have been introduced.
     A pleasure to meet you. I am C-3p0, JSON-DB Relations."

Do you think JSON is excellent, but it could be better handled in your DB code?

whether you like [tokio-postgres](https://crates.io/crates/tokio-postgres) or 
[Sqlx](https://crates.io/crates/sqlx), 
C3p0 brings you a set of tools to simplify JSON integration in your database workflow.

So, if you would like to be able to fetch/delete/insert/update JSON object interactively with your Sql DB like it was a NoSQL DB, then keep reading!


## What C3p0 is not

Although it provides a high-level interface for basic CRUD operations, _C3p0_ is neither an ORM nor a replacement for one—or for any similar tool. In fact, _C3p0_ is not an ORM at all. It allows storing and retrieving JSON objects from the database, but it does not manage cross-table relationships. Each object type is stored in its own dedicated table, using a single column of JSON type.

__C3p0__: "I see, Sir Luke".

Great!


## What C3p0 is

_C3p0_ is a library designed for integrating JSON data with relational databases. It offers the following capabilities:
- Performs basic CRUD operations on JSON objects
- Automatically generates the necessary SQL queries to interact with database tables, without relying on macros
- Supports PostgreSQL (via `tokio-postgres` or `sqlx`), as well as MySQL and SQLite (via `sqlx`)


## Prerequisites

It uses async closures, so it requires at least Rust version 1.85.


## History
The first _C3p0_ version was written in Java...

__C3p0__: "If I told you half the things I’ve heard about this Jabba the Hutt, you’d probably short circuit."

I said "Java", "Ja"-"va". Stay focused, please!

Anyway, Java is slowly showing its age, and we got a bit bored about it.

__C3p0__: "they're using a very primitive dialect".

Indeed.

On the contrary, our interest in the Rust programming language has kept growing over time;
so, we experimented with it more and more and, finally, migrated some critical portions of our code to Rust.

Just said, we love it.

We believe that Rust is a better overall language.

__C3p0__: "The city's central computer told you?"
 
Yes! It allows us to achieve better resource usage, to avoid the garbage collector and the virtual machine,
and, at the same time, to get a better and safer concurrency level.


## Can I use it in production?
__Han__ "Don't worry. Everything's gonna be fine. Trust me."

__C3p0__: "Every time he employs that phrase, my circuitry becomes erratic!"

__Han__: ???

__C3p0__: "Artoo says that the chances of survival are 725 to 1".

By the way, regardless of what Artoo said, we've survived using it in production since 2018.


## Example of usage

Here is an example of how to use _C3p0_ with sqlx and a Postgres database:

```rust
use c3p0::sqlx::{
    SqlxPgC3p0Pool,
    sqlx::{PgPool, postgres::PgConnectOptions},
};
use serde::{Deserialize, Serialize};
use testcontainers::{
    postgres::Postgres,
    testcontainers::{ContainerAsync, runners::AsyncRunner},
};

/// Starts a Postgres container and returns a SqlxPgC3p0Pool connection pool for it.    
pub async fn create_c3p0_pool() -> (SqlxPgC3p0Pool, ContainerAsync<Postgres>) {
    // starts a Postgres container for testing
    let node = Postgres::default()
        .start()
        .await
        .expect("Could not start container");

    // Create a Sqlx connection pool for the Postgres database
    let pool = PgPool::connect_with(
        PgConnectOptions::new()
            .username("postgres")
            .password("postgres")
            .database("postgres")
            .host("127.0.0.1")
            .port(node.get_host_port_ipv4(5432).await.unwrap()),
    )
    .await
    .unwrap();

    // Create a C3p0 pool from the Sqlx pool
    let pool = SqlxPgC3p0Pool::new(pool);

    (pool, node)
}

/// Example of a model for a database table
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct UserData {
    pub username: String,
    pub email: String,
}

/// Example of how to use c3p0 with sqlx and a Postgres database
#[tokio::test]
async fn postgres_sqlx_example() {
    use c3p0::{sqlx::SqlxPgC3p0JsonBuilder, *};

    let (c3p0, _pg) = create_c3p0_pool().await;

    // Create a C3p0Json to manage the UserData model using the table "user_data"
    let jpo = SqlxPgC3p0JsonBuilder::new("user_data").build::<UserData>();

    // Open a transaction to the database.
    // C3p0 will commit or rollback the transaction automatically. 
    // The transaction will be committed if the result is Ok, otherwise it will be rolled back.
    let result: Result<_, C3p0Error> = c3p0
        .transaction(async |tx| {
            // Create the table if it doesn't exist. Usually this would be done in a migration
            assert!(jpo.create_table_if_not_exists(tx).await.is_ok());
            println!("Table created!");

            // Create a new UserData object
            let user_data = UserData {
                username: "Francesco Cina".to_string(),
                email: "ufoscout@ufoscout.com".to_string(),
            };

            // Save the new UserData object to the database
            let create_user = jpo.save(tx, user_data.into()).await.unwrap();
            println!("Saved user data: {:?}", create_user);

            // Get the saved UserData object from the database
            let fetch_user = jpo.fetch_one_by_id(tx, &create_user.id).await.unwrap();
            assert_eq!(fetch_user, create_user);

            // Delete the saved UserData object from the database
            let deleted_rows_count = jpo.delete_by_id(tx, &create_user.id).await.unwrap();
            assert_eq!(deleted_rows_count, 1);

            // Count the number of UserData objects in the database
            let count = jpo.count_all(tx).await.unwrap();
            assert_eq!(count, 0);

            // delete the table
            jpo.drop_table_if_exists(tx, true).await.unwrap();

            Ok(())
        })
        .await;

    assert!(result.is_ok());
}

```
