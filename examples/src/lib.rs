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
