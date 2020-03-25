use crate::utils::*;
use crate::*;

#[test]
fn json_should_commit_transaction() {
    let data = data(false);
    let pool = &data.0;
    let c3p0: C3p0Impl = pool.clone();
    let table_name = format!("TEST_TABLE_{}", rand_string(8));
    let jpo = C3p0JsonBuilder::new(table_name).build();

    let model = NewModel::new(TestData {
        first_name: "my_first_name".to_owned(),
        last_name: "my_last_name".to_owned(),
    });

    let result: Result<_, C3p0Error> = c3p0.transaction(|conn| {
        assert!(jpo.create_table_if_not_exists(conn).is_ok());
        assert!(jpo.save(conn, model.clone()).is_ok());
        assert!(jpo.save(conn, model.clone()).is_ok());
        assert!(jpo.save(conn, model.clone()).is_ok());
        Ok(())
    });

    assert!(result.is_ok());

    {
        let conn = &mut c3p0.connection().unwrap();
        let count = jpo.count_all(conn).unwrap();
        assert_eq!(3, count);

        assert!(jpo.drop_table_if_exists(conn, true).is_ok());
    }
}

#[test]
fn should_rollback_transaction() {
    let data = data(false);
    let pool = &data.0;
    let c3p0: C3p0Impl = pool.clone();
    let table_name = format!("TEST_TABLE_{}", rand_string(8));
    let jpo = C3p0JsonBuilder::new(table_name).build();

    let model = NewModel::new(TestData {
        first_name: "my_first_name".to_owned(),
        last_name: "my_last_name".to_owned(),
    });

    let result_create_table: Result<(), C3p0Error> = c3p0.transaction(|conn| {
        assert!(jpo.create_table_if_not_exists(conn).is_ok());
        Ok(())
    });
    assert!(result_create_table.is_ok());

    let result: Result<(), C3p0Error> = c3p0.transaction(|conn| {
        assert!(jpo.save(conn, model.clone()).is_ok());
        assert!(jpo.save(conn, model.clone()).is_ok());
        assert!(jpo.save(conn, model.clone()).is_ok());
        Err(C3p0Error::ResultNotFoundError)?
    });

    assert!(result.is_err());

    {
        let conn = &mut c3p0.connection().unwrap();
        let count = jpo.count_all(conn).unwrap();
        assert_eq!(0, count);

        assert!(jpo.drop_table_if_exists(conn, true).is_ok());
    }
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

    let data = data(false);
    let pool = &data.0;
    let c3p0: C3p0Impl = pool.clone();

    let result: Result<(), _> = c3p0.transaction(|_| Err(CustomError::InnerError));

    assert!(result.is_err());

    match &result {
        Err(CustomError::InnerError) => assert!(true),
        _ => assert!(false),
    }
}
