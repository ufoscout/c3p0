use c3p0::{
    sqlx::{postgres::PgConnectOptions, PgPool}, C3p0Error, C3p0Pool, Data, PgC3p0Pool, Tx
};
use serde::{Deserialize, Serialize};
use testcontainers::{
    postgres::Postgres,
    testcontainers::{ContainerAsync, runners::AsyncRunner},
};

/// Starts a Postgres container and returns a SqlxPgC3p0Pool connection pool for it.    
pub async fn create_c3p0_pool() -> (PgC3p0Pool, ContainerAsync<Postgres>) {
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
    let pool = PgC3p0Pool::new(pool);

    (pool, node)
}

/// Example of a model for a database table
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct UserData {
    pub username: String,
    pub email: String,
}

/// Implement the Data trait for the UserData model using the table "user_data"
impl Data for UserData {
    const TABLE_NAME: &'static str = "user_data";
    type CODEC = Self;
}

/// Example of how to use c3p0 with sqlx and a Postgres database
#[tokio::main]
async fn main() {

    let (c3p0, _pg) = create_c3p0_pool().await;

    // Open a transaction to the database.
    // C3p0 will commit or rollback the transaction automatically.
    // The transaction will be committed if the result is Ok, otherwise it will be rolled back.
    let result: Result<_, C3p0Error> = c3p0
        .transaction(async |tx| {
            // Create the table if it doesn't exist. Usually this would be done in a migration
            assert!(tx.create_table_if_not_exists::<UserData>().await.is_ok());
            println!("Table created!");

            // Create a new UserData object
            let user_data = UserData {
                username: "Francesco Cina".to_string(),
                email: "ufoscout@ufoscout.com".to_string(),
            };

            // Save the new UserData object to the database
            let create_user = tx.save(user_data.into()).await.unwrap();
            println!("Saved user data: {create_user:?}");

            // Get the saved UserData object from the database
            let fetch_user = tx.fetch_one_by_id::<UserData>(create_user.id).await.unwrap();
            assert_eq!(fetch_user, create_user);

            // Delete the saved UserData object from the database
            let deleted_rows_count = tx.delete_by_id::<UserData>(create_user.id).await.unwrap();
            assert_eq!(deleted_rows_count, 1);

            // Count the number of UserData objects in the database
            let count = tx.count_all::<UserData>().await.unwrap();
            assert_eq!(count, 0);

            // delete the table
            tx.drop_table_if_exists::<UserData>(true).await.unwrap();

            Ok(())
        })
        .await;

    assert!(result.is_ok());
}
