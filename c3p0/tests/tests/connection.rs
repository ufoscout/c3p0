use crate::utils::*;
use crate::*;

#[test]
fn should_execute_and_fetch() -> Result<(), C3p0Error> {
    test(async {
        let data = data(false).await;
        let pool = &data.0;

        pool.transaction(async |conn| {
            let table_name = format!("TEST_TABLE_{}", rand_string(8));

            assert!(
                conn.execute(
                    &format!(
                        r"CREATE TABLE {table_name} (
                             name varchar(255)
                          )"
                    ),
                    &[]
                )
                .await
                .is_ok()
            );

            assert_eq!(
                0,
                conn.fetch_one_value::<i64>(&format!("SELECT COUNT(*) FROM {table_name}"), &[])
                    .await
                    .unwrap()
            );

            let insert = &db_specific::build_insert_query(&table_name);

            assert_eq!(1, conn.execute(insert, &[&"one"]).await.unwrap());

            assert_eq!(
                1,
                conn.fetch_one_value::<i64>(&format!("SELECT COUNT(*) FROM {table_name}"), &[])
                    .await
                    .unwrap()
            );

            let fetch_result_1 = conn
                .fetch_one(
                    &format!(r"SELECT * FROM {table_name} WHERE name = 'one'"),
                    &[],
                    db_specific::row_to_string,
                )
                .await;
            assert!(fetch_result_1.is_ok());
            assert_eq!("one".to_owned(), fetch_result_1.unwrap());

            let fetch_result_2 = conn
                .fetch_one(
                    &format!(r"SELECT * FROM {table_name} WHERE name = 'two'"),
                    &[],
                    db_specific::row_to_string,
                )
                .await;
            assert!(fetch_result_2.is_err());

            assert!(
                conn.execute(&format!(r"DROP TABLE {table_name}"), &[])
                    .await
                    .is_ok()
            );
            Ok(())
        })
        .await
    })
}

#[test]
fn should_execute_and_fetch_option() -> Result<(), C3p0Error> {
    test(async {
        let data = data(false).await;
        let pool = &data.0;

        pool.transaction(async |conn| {
            let table_name = format!("TEST_TABLE_{}", rand_string(8));
            assert!(
                conn.execute(
                    &format!(
                        r"CREATE TABLE {table_name} (
                             name varchar(255)
                          )"
                    ),
                    &[]
                )
                .await
                .is_ok()
            );

            let insert = &db_specific::build_insert_query(&table_name);

            assert_eq!(1, conn.execute(insert, &[&"one"]).await.unwrap());

            let fetch_result = conn
                .fetch_one_optional(
                    &format!(r"SELECT * FROM {table_name} WHERE name = 'one'"),
                    &[],
                    db_specific::row_to_string,
                )
                .await;
            assert!(fetch_result.is_ok());
            assert_eq!(Some("one".to_owned()), fetch_result.unwrap());

            let fetch_result = conn
                .fetch_one_optional(
                    &format!(r"SELECT * FROM {table_name} WHERE name = 'two'"),
                    &[],
                    db_specific::row_to_string,
                )
                .await;
            assert!(fetch_result.is_ok());
            assert_eq!(None, fetch_result.unwrap());

            assert!(
                conn.execute(&format!(r"DROP TABLE {table_name}"), &[])
                    .await
                    .is_ok()
            );
            Ok(())
        })
        .await
    })
}

#[test]
fn should_batch_execute() -> Result<(), C3p0Error> {
    test(async {
        let data = data(false).await;
        let pool = &data.0;
        pool.transaction(async |conn| {
            let table_name = format!("TEST_TABLE_{}", rand_string(8));

            let insert = &format!(
                r"
                CREATE TABLE {table_name} ( name varchar(255) );
                INSERT INTO {table_name} (name) VALUES ('new-name-1');
                INSERT INTO {table_name} (name) VALUES ('new-name-2');
                DROP TABLE {table_name};
        "
            );

            assert!(conn.batch_execute(insert).await.is_ok());
            Ok(())
        })
        .await
    })
}

#[test]
fn should_fetch_values() -> Result<(), C3p0Error> {
    test(async {
        let data = data(false).await;
        let pool = &data.0;

        let table_name = &format!("TEST_TABLE_{}", rand_string(8));

        let result: Result<_, C3p0Error> = pool
            .transaction(async |conn| {
                assert!(
                    conn.batch_execute(&format!(
                        "CREATE TABLE {table_name} ( name varchar(255) )"
                    ))
                    .await
                    .is_ok()
                );

                let all_string = conn
                    .fetch_all_values::<String>(&format!("SELECT * FROM {table_name}"), &[])
                    .await;
                assert!(all_string.is_ok());
                assert!(all_string.unwrap().is_empty());

                let one_string = conn
                    .fetch_one_value::<String>(&format!("SELECT * FROM {table_name}"), &[])
                    .await;
                assert!(one_string.is_err());

                let one_i64 = conn
                    .fetch_one_value::<i64>(&format!("SELECT * FROM {table_name}"), &[])
                    .await;
                assert!(one_i64.is_err());

                conn.batch_execute(&format!(
                    r"INSERT INTO {table_name} (name) VALUES ('one');
                                    INSERT INTO {table_name} (name) VALUES ('two');
                                    INSERT INTO {table_name} (name) VALUES ('three')"
                ))
                .await
                .unwrap();

                let all_string = conn
                    .fetch_all_values::<String>(
                        &format!("SELECT name FROM {table_name} order by name"),
                        &[],
                    )
                    .await;
                assert!(all_string.is_ok());
                assert_eq!(
                    vec!["one".to_owned(), "three".to_owned(), "two".to_owned(),],
                    all_string.unwrap()
                );

                let all_i64 = conn
                    .fetch_all_values::<i64>(&format!("SELECT * FROM {table_name}"), &[])
                    .await;
                assert!(all_i64.is_err());

                let one_string = conn
                    .fetch_one_value::<String>(
                        &format!("SELECT name FROM {table_name} order by name"),
                        &[],
                    )
                    .await;
                assert!(one_string.is_ok());
                assert_eq!("one".to_owned(), one_string.unwrap());

                let one_i64 = conn
                    .fetch_one_value::<i64>(&format!("SELECT * FROM {table_name}"), &[])
                    .await;
                assert!(one_i64.is_err());

                assert!(
                    conn.batch_execute(&format!("DROP TABLE {table_name}"))
                        .await
                        .is_ok()
                );

                Ok(())
            })
            .await;

        assert!(result.is_ok());

        Ok(())
    })
}
