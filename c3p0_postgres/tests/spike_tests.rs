#![cfg(feature = "spike_tests")]

use testcontainers::*;
use tokio_postgres::NoTls;

#[tokio::test]
async fn should_cast_parameter() {
    let docker = clients::Cli::default();
    let node = docker.run(images::postgres::Postgres::default());

    // Connect to the database.
    let (client, connection) = tokio_postgres::connect(
        &format!(
            "host=localhost port={} user=postgres password=postgres dbname=postgres",
            node.get_host_port(5432).unwrap()
        ),
        NoTls,
    )
    .await
    .unwrap();

    // The connection object performs the actual communication with the database,
    // so spawn it off to run on its own.
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    client
        .batch_execute(
            r#"
    create  table IF NOT EXISTS MY_TABLE (
        ID bigserial primary key
    );
    "#,
        )
        .await
        .unwrap();

    // OK
    client
        .query("SELECT ID FROM MY_TABLE WHERE ID = '123'::bigint", &[])
        .await
        .unwrap();

    // Ok
    client
        .query(
            "SELECT ID FROM MY_TABLE WHERE ID = $1::text::bigint",
            &[&"123"],
        )
        .await
        .unwrap();

    // Error { kind: ToSql(0), cause: Some(WrongType { postgres: Int8, rust: "&str" }) }
    //client.query("SELECT ID FROM MY_TABLE WHERE ID = $1::bigint", &[&"123"]).await.unwrap();
}
