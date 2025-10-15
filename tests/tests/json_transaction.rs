use serde::{Deserialize, Serialize};

use crate::utils::*;
use crate::*;

#[test]
fn json_should_commit_transaction() {
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub struct TestData {
        pub first_name: String,
        pub last_name: String,
    }

    impl c3p0::DataType for TestData {
        const TABLE_NAME: &'static str =
            const_format::concatcp!("TEST_TABLE_", const_random::const_random!(u64));
        type CODEC = Self;
    }

    run_test(async {
        let data = data(false).await;
        let c3p0 = &data.0;

        let model = NewRecord::new(TestData {
            first_name: "my_first_name".to_owned(),
            last_name: "my_last_name".to_owned(),
        });

        let model_clone = model.clone();

        let result: Result<(), C3p0Error> = c3p0
            .transaction(async |conn| {
                assert!(conn.create_table_if_not_exists::<TestData>().await.is_ok());
                println!("Table created!");
                assert!(conn.save(model.clone()).await.is_ok());
                println!("Record saved 1!");
                assert!(conn.save(model.clone()).await.is_ok());
                println!("Record saved 2!");
                assert!(conn.save(model_clone.clone()).await.is_ok());
                println!("Record saved 3!");
                Ok(())
            })
            .await;

        assert!(result.is_ok());

        c3p0.transaction::<_, C3p0Error, _>(async |conn| {
            let count = conn.count_all::<TestData>().await.unwrap();
            assert_eq!(3, count);
            println!("Count performed!");

            // It should be possible to query with both the Record and the DataType
            let count = conn.count_all::<Record<TestData>>().await.unwrap();
            assert_eq!(3, count);
            println!("Count performed!");

            // It should be possible to query with both the NewRecord and the DataType
            let count = conn.count_all::<NewRecord<TestData>>().await.unwrap();
            assert_eq!(3, count);
            println!("Count performed!");

            let _ = conn.drop_table_if_exists::<TestData>(true).await;
            println!("Table dropped!");
            Ok(())
        })
        .await
        .unwrap();
    })
}

#[test]
fn json_should_rollback_transaction() {
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub struct TestData {
        pub first_name: String,
        pub last_name: String,
    }

    impl c3p0::DataType for TestData {
        const TABLE_NAME: &'static str =
            const_format::concatcp!("TEST_TABLE_", const_random::const_random!(u64));
        type CODEC = Self;
    }

    run_test(async {
        let data = data(false).await;
        let c3p0 = &data.0;

        let model = NewRecord::new(TestData {
            first_name: "my_first_name".to_owned(),
            last_name: "my_last_name".to_owned(),
        });

        let result_create_table: Result<(), C3p0Error> = c3p0
            .transaction(async |conn| {
                assert!(conn.create_table_if_not_exists::<TestData>().await.is_ok());
                Ok(())
            })
            .await;
        assert!(result_create_table.is_ok());

        let result: Result<(), C3p0Error> = c3p0
            .transaction(async |conn| {
                assert!(conn.save(model.clone()).await.is_ok());
                assert!(conn.save(model.clone()).await.is_ok());
                assert!(conn.save(model.clone()).await.is_ok());
                Err(C3p0Error::ResultNotFoundError)?
            })
            .await;

        assert!(result.is_err());

        {
            c3p0.transaction::<_, C3p0Error, _>(async |conn| {
                let count = conn.count_all::<TestData>().await.unwrap();
                assert_eq!(0, count);

                let _ = conn.drop_table_if_exists::<TestData>(true).await;
                Ok(())
            })
            .await
            .unwrap();
        }
    });
}

#[test]
fn json_transaction_should_return_internal_error() {
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

    run_test(async {
        let data = data(false).await;
        let c3p0 = &data.0;

        let result: Result<(), _> = c3p0
            .transaction(async |_| Err(CustomError::InnerError))
            .await;

        assert!(result.is_err());

        match &result {
            Err(CustomError::InnerError) => (),
            _ => panic!(),
        }
    });
}
