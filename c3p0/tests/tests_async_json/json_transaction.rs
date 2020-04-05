use crate::utils::*;
use crate::*;

#[test]
fn json_should_commit_transaction() {
    test(async {
        let data = data(false).await;
        let c3p0 = &data.0;

        let table_name = format!("TEST_TABLE_{}", rand_string(8));
        let jpo = C3p0JsonBuilder::new(table_name).build::<TestData>();

        let model = NewModel::new(TestData {
            first_name: "my_first_name".to_owned(),
            last_name: "my_last_name".to_owned(),
        });

        let model_clone = model.clone();

        let jpo_ref = &jpo;
        let result: Result<(), C3p0Error> = c3p0
            .transaction(|mut conn| async move {
                let conn = &mut conn;
                assert!(jpo_ref.create_table_if_not_exists(conn).await.is_ok());
                assert!(jpo_ref.save(conn, model.clone()).await.is_ok());
                assert!(jpo_ref.save(conn, model.clone()).await.is_ok());
                assert!(jpo_ref.save(conn, model_clone.clone()).await.is_ok());
                Ok(())
            })
            .await;

        assert!(result.is_ok());

        c3p0.transaction::<_, C3p0Error, _, _>(|mut conn| async move {
            let conn = &mut conn;
            let count = jpo.count_all(conn).await.unwrap();
            assert_eq!(3, count);

            assert!(jpo.drop_table_if_exists(conn, true).await.is_ok());
            Ok(())
        })
        .await
        .unwrap();
    })
}

#[test]
fn should_rollback_transaction() {
    test(async {
        let data = data(false).await;
        let c3p0 = &data.0;

        let table_name = format!("TEST_TABLE_{}", rand_string(8));
        let jpo = &C3p0JsonBuilder::new(table_name).build();

        let model = NewModel::new(TestData {
            first_name: "my_first_name".to_owned(),
            last_name: "my_last_name".to_owned(),
        });

        let result_create_table: Result<(), C3p0Error> = c3p0
            .transaction(|mut conn| async move {
                assert!(jpo.create_table_if_not_exists(&mut conn).await.is_ok());
                Ok(())
            })
            .await;
        assert!(result_create_table.is_ok());

        let result: Result<(), C3p0Error> = c3p0
            .transaction(|mut conn| async move {
                assert!(jpo.save(&mut conn, model.clone()).await.is_ok());
                assert!(jpo.save(&mut conn, model.clone()).await.is_ok());
                assert!(jpo.save(&mut conn, model.clone()).await.is_ok());
                Err(C3p0Error::ResultNotFoundError)?
            })
            .await;

        assert!(result.is_err());

        {
            c3p0.transaction::<_, C3p0Error, _, _>(|mut conn| async move {
                let conn = &mut conn;
                let count = jpo.count_all(conn).await.unwrap();
                assert_eq!(0, count);

                assert!(jpo.drop_table_if_exists(conn, true).await.is_ok());
                Ok(())
            })
            .await
            .unwrap();
        }
    });
}

#[test]
fn transaction_should_return_internal_error() {
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

    test(async {
        let data = data(false).await;
        let c3p0 = &data.0;

        let result: Result<(), _> = c3p0
            .transaction(|_| async { Err(CustomError::InnerError) })
            .await;

        assert!(result.is_err());

        match &result {
            Err(CustomError::InnerError) => assert!(true),
            _ => assert!(false),
        }
    });
}
