use crate::utils::*;
use crate::*;

#[test]
fn should_commit_transaction() {
    test(async {
        let data = data(false).await;
        let pool = &data.0;
        let c3p0: C3p0Impl = pool.clone();
        let table_name = &format!("TEST_TABLE_{}", rand_string(8));

        let result: Result<_, C3p0Error> = c3p0
            .transaction(async |conn| {
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

                conn.execute(
                    &format!(r"INSERT INTO {table_name} (name) VALUES ('one')"),
                    &[],
                )
                .await
                .unwrap();
                conn.execute(
                    &format!(r"INSERT INTO {table_name} (name) VALUES ('two')"),
                    &[],
                )
                .await
                .unwrap();
                conn.execute(
                    &format!(r"INSERT INTO {table_name} (name) VALUES ('three')"),
                    &[],
                )
                .await
                .unwrap();
                Ok(())
            })
            .await;

        assert!(result.is_ok());

        {
            pool.transaction::<_, C3p0Error, _>(async |conn| {
                let count = conn
                    .fetch_one_value::<i64>(&format!(r"SELECT COUNT(*) FROM {table_name}"), &[])
                    .await
                    .unwrap();
                assert_eq!(3, count);

                assert!(
                    conn.execute(&format!(r"DROP TABLE {table_name}"), &[])
                        .await
                        .is_ok()
                );
                Ok(())
            })
            .await
            .unwrap();
        }
    })
}

#[test]
fn should_rollback_transaction() {
    test(async {
        let data = data(false).await;
        let pool = &data.0;
        let c3p0: C3p0Impl = pool.clone();
        let table_name = &format!("TEST_TABLE_{}", rand_string(8));

        let result_create_table: Result<(), C3p0Error> = c3p0
            .transaction(async |conn| {
                assert!(
                    conn.batch_execute(&format!(
                        r"CREATE TABLE {table_name} (
                             name varchar(255)
                          )"
                    ))
                    .await
                    .is_ok()
                );
                Ok(())
            })
            .await;
        assert!(result_create_table.is_ok());

        let result: Result<(), C3p0Error> = c3p0
            .transaction(async |conn| {
                conn.execute(
                    &format!(r"INSERT INTO {table_name} (name) VALUES ('one')"),
                    &[],
                )
                .await
                .unwrap();
                conn.execute(
                    &format!(r"INSERT INTO {table_name} (name) VALUES ('two')"),
                    &[],
                )
                .await
                .unwrap();
                conn.execute(
                    &format!(r"INSERT INTO {table_name} (name) VALUES ('three')"),
                    &[],
                )
                .await
                .unwrap();
                Err(C3p0Error::ResultNotFoundError)?
            })
            .await;

        assert!(result.is_err());

        {
            pool.transaction::<_, C3p0Error, _>(async |conn| {
                let count = conn
                    .fetch_one_value::<i64>(&format!(r"SELECT COUNT(*) FROM {table_name}"), &[])
                    .await
                    .unwrap();
                assert_eq!(0, count);

                assert!(
                    conn.execute(&format!(r"DROP TABLE IF EXISTS {table_name}"), &[])
                        .await
                        .is_ok()
                );
                Ok(())
            })
            .await
            .unwrap();
        }
    })
}

#[test]
fn transaction_should_return_internal_error() {
    test(async {
        use thiserror::Error;

        #[derive(Error, Debug, PartialEq)]
        pub enum CustomError {
            #[error("InnerError")]
            InnerError,
            #[error("C3p0Error")]
            C3p0Error,
        }

        impl From<C3p0Error> for CustomError {
            fn from(_: C3p0Error) -> Self {
                CustomError::C3p0Error
            }
        }

        let data = data(false).await;
        let pool = &data.0;
        let c3p0: C3p0Impl = pool.clone();

        let result: Result<(), _> = c3p0
            .transaction(async |_| Err(CustomError::InnerError))
            .await;

        assert!(result.is_err());

        match &result {
            Err(CustomError::InnerError) => (),
            _ => panic!(),
        }
    })
}
