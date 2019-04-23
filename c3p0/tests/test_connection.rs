use c3p0::prelude::*;

#[cfg(feature = "pg")]
mod shared_pg;
#[cfg(feature = "pg")]
use crate::shared_pg::*;

#[cfg(feature = "mysql")]
mod shared_mysql;
#[cfg(feature = "mysql")]
use crate::shared_mysql::*;
use c3p0::client::Row;

#[test]
fn should_execute_and_fetch() {
    SINGLETON.get(|(pool, _)| {
        let pool = pool.clone();

        let c3p0 = C3p0Builder::build(pool);

        let conn = c3p0.connection().unwrap();

        assert!(conn
            .execute(
                r"CREATE TABLE TEST_TABLE (
                             name varchar(255)
                          )",
                &[]
            )
            .is_ok());

        assert_eq!(
            0,
            conn.fetch_one_value::<i64>("SELECT COUNT(*) FROM TEST_TABLE", &[])
                .unwrap()
        );

        #[cfg(feature = "pg")]
        let insert = r"INSERT INTO TEST_TABLE (name) VALUES ($1)";
        #[cfg(feature = "mysql")]
        let insert = r"INSERT INTO TEST_TABLE (name) VALUES (?)";

        assert_eq!(1, conn.execute(insert, &[&"one"]).unwrap());

        assert_eq!(
            1,
            conn.fetch_one_value::<i64>("SELECT COUNT(*) FROM TEST_TABLE", &[])
                .unwrap()
        );

        #[cfg(feature = "pg")]
        let mapper = |row: &Row| {
            let value: String = row.get(0);
            Ok(value)
        };
        #[cfg(feature = "mysql")]
        let mapper = |row: &Row| {
            let value: String = row.get(0).ok_or_else(|| C3p0Error::ResultNotFoundError)?;
            Ok(value)
        };

        let fetch_result_1 =
            conn.fetch_one(r"SELECT * FROM TEST_TABLE WHERE name = 'one'", &[], mapper);
        assert!(fetch_result_1.is_ok());
        assert_eq!("one".to_owned(), fetch_result_1.unwrap());

        let fetch_result_2 =
            conn.fetch_one(r"SELECT * FROM TEST_TABLE WHERE name = 'two'", &[], mapper);
        assert!(fetch_result_2.is_err());

        assert!(conn.execute(r"DROP TABLE TEST_TABLE", &[]).is_ok());
    });
}

#[test]
fn should_execute_and_fetch_option() {
    SINGLETON.get(|(pool, _)| {
        let pool = pool.clone();

        let c3p0 = C3p0Builder::build(pool);

        let conn = c3p0.connection().unwrap();

        assert!(conn
            .execute(
                r"CREATE TABLE TEST_TABLE (
                             name varchar(255)
                          )",
                &[]
            )
            .is_ok());

        #[cfg(feature = "pg")]
        let insert = r"INSERT INTO TEST_TABLE (name) VALUES ($1)";
        #[cfg(feature = "mysql")]
        let insert = r"INSERT INTO TEST_TABLE (name) VALUES (?)";

        assert_eq!(1, conn.execute(insert, &[&"one"]).unwrap());

        #[cfg(feature = "pg")]
        let mapper = |row: &Row| Ok(row.get(0));
        #[cfg(feature = "mysql")]
        let mapper = |row: &Row| Ok(row.get(0).ok_or_else(|| C3p0Error::ResultNotFoundError)?);

        let fetch_result =
            conn.fetch_one_option(r"SELECT * FROM TEST_TABLE WHERE name = 'one'", &[], mapper);
        assert!(fetch_result.is_ok());
        assert_eq!(Some("one".to_owned()), fetch_result.unwrap());

        let fetch_result =
            conn.fetch_one_option(r"SELECT * FROM TEST_TABLE WHERE name = 'two'", &[], mapper);
        assert!(fetch_result.is_ok());
        assert_eq!(None, fetch_result.unwrap());

        assert!(conn.execute(r"DROP TABLE TEST_TABLE", &[]).is_ok());
    });
}

#[test]
fn should_batch_execute() {
    SINGLETON.get(|(pool, _)| {
        let pool = pool.clone();

        let c3p0 = C3p0Builder::build(pool);
        let conn = c3p0.connection().unwrap();

        let insert = r"
                CREATE TABLE TEST_TABLE ( name varchar(255) );
                INSERT INTO TEST_TABLE (name) VALUES ('new-name-1');
                INSERT INTO TEST_TABLE (name) VALUES ('new-name-2');
                DROP TABLE TEST_TABLE;
        ";

        assert!(conn.batch_execute(insert).is_ok());
    });
}

#[test]
fn should_fetch_all() {
    SINGLETON.get(|(pool, _)| {
        let pool = pool.clone();

        let c3p0 = C3p0Builder::build(pool);

        c3p0.transaction(|conn| {
            assert!(conn
                .batch_execute("CREATE TABLE TEST_TABLE ( name varchar(255) )")
                .is_ok());

            let all_string = conn.fetch_all_values::<String>("SELECT * FROM TEST_TABLE", &[]);
            assert!(all_string.is_ok());
            assert!(all_string.unwrap().is_empty());

            assert!(conn
                .execute(r"INSERT INTO TEST_TABLE (name) VALUES ('one')", &[])
                .is_ok());
            assert!(conn
                .execute(r"INSERT INTO TEST_TABLE (name) VALUES ('two')", &[])
                .is_ok());
            assert!(conn
                .execute(r"INSERT INTO TEST_TABLE (name) VALUES ('three')", &[])
                .is_ok());

            let all_string = conn.fetch_all_values::<String>("SELECT * FROM TEST_TABLE", &[]);
            assert_eq!(3, all_string.unwrap().len());
            //assert!(all_string.is_ok());

            let all_i64 = conn.fetch_all_values::<i64>("SELECT * FROM TEST_TABLE", &[]);
            assert!(all_i64.is_err());

            assert!(conn.batch_execute("DROP TABLE TEST_TABLE").is_ok());

            Ok(())
        })
        .unwrap();
    });
}
