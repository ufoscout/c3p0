use c3p0::prelude::*;

#[cfg(feature = "pg")]
mod shared_pg;
#[cfg(feature = "pg")]
use crate::shared_pg::*;

#[cfg(feature = "mysql")]
mod shared_mysql;
#[cfg(feature = "mysql")]
use crate::shared_mysql::*;

#[test]
fn should_commit_transaction() {
    SINGLETON.get(|(pool, _)| {
        let c3p0: C3p0 = pool.clone();

        let conn = c3p0.connection().unwrap();

        assert!(conn
            .execute(
                r"CREATE TABLE TEST_TABLE (
                             name varchar(255)
                          )",
                &[]
            )
            .is_ok());

        let result: Result<(), C3p0Error> = c3p0.transaction(|conn| {
            conn.execute(r"INSERT INTO TEST_TABLE (name) VALUES ('one')", &[])
                .unwrap();
            conn.execute(r"INSERT INTO TEST_TABLE (name) VALUES ('two')", &[])
                .unwrap();
            conn.execute(r"INSERT INTO TEST_TABLE (name) VALUES ('three')", &[])
                .unwrap();
            Ok(())
        });

        assert!(result.is_ok());

        let count = conn
            .fetch_one_value::<i64>(r"SELECT COUNT(*) FROM TEST_TABLE", &[])
            .unwrap();
        assert_eq!(3, count);

        assert!(conn.execute(r"DROP TABLE TEST_TABLE", &[]).is_ok());
    });
}

#[test]
fn should_rollback_transaction() {
    SINGLETON.get(|(pool, _)| {
        let c3p0: C3p0 = pool.clone();

        let conn = c3p0.connection().unwrap();

        assert!(conn
            .execute(
                r"CREATE TABLE TEST_TABLE (
                             name varchar(255)
                          )",
                &[]
            )
            .is_ok());

        let result: Result<(), C3p0Error> = c3p0.transaction(|conn| {
            conn.execute(r"INSERT INTO TEST_TABLE (name) VALUES ('one')", &[])
                .unwrap();
            conn.execute(r"INSERT INTO TEST_TABLE (name) VALUES ('two')", &[])
                .unwrap();
            conn.execute(r"INSERT INTO TEST_TABLE (name) VALUES ('three')", &[])
                .unwrap();
            Err(C3p0Error::ResultNotFoundError)?
        });

        assert!(result.is_err());

        let count = conn
            .fetch_one_value::<i64>(r"SELECT COUNT(*) FROM TEST_TABLE", &[])
            .unwrap();
        assert_eq!(0, count);

        assert!(conn.execute(r"DROP TABLE TEST_TABLE", &[]).is_ok());
    });
}
