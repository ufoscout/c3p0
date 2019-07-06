use c3p0_json::*;
use crate::*;
use crate::tests::util::rand_string;

#[test]
fn should_execute_and_fetch() {
    SINGLETON.get(|(pool, _)| {
        let c3p0: C3p0Impl = pool.clone();

        let conn = c3p0.connection().unwrap();

        let table_name = format!("TEST_TABLE_{}", rand_string(8));

        assert!(conn
            .execute(
                &format!(r"CREATE TABLE {} (
                             name varchar(255)
                          )", table_name),
                &[]
            )
            .is_ok());

        assert_eq!(
            0,
            conn.fetch_one_value::<i64>(&format!("SELECT COUNT(*) FROM {}", table_name), &[])
                .unwrap()
        );

        #[cfg(feature = "pg")]
        let insert = &format!(r"INSERT INTO {} (name) VALUES ($1)", table_name);
        #[cfg(any(feature = "mysql", feature = "sqlite"))]
        let insert = &format!(r"INSERT INTO {} (name) VALUES (?)", table_name);

        assert_eq!(1, conn.execute(insert, &[&"one"]).unwrap());

        assert_eq!(
            1,
            conn.fetch_one_value::<i64>(&format!("SELECT COUNT(*) FROM {}", table_name), &[])
                .unwrap()
        );

        #[cfg(feature = "pg")]
        let mapper = |row: &Row| {
            let value: String = row.get(0);
            Ok(value)
        };
        #[cfg(feature = "mysql")]
        let mapper = |row: &Row| {
            let value: String = row
                .get(0)
                .ok_or_else(|| c3p0_json::C3p0Error::ResultNotFoundError)?;
            Ok(value)
        };
        #[cfg(feature = "sqlite")]
        let mapper = |row: &Row| {
            let value: String = row.get(0)?;
            Ok(value)
        };
        let fetch_result_1 =
            conn.fetch_one(&format!(r"SELECT * FROM {} WHERE name = 'one'", table_name), &[], mapper);
        assert!(fetch_result_1.is_ok());
        assert_eq!("one".to_owned(), fetch_result_1.unwrap());

        let fetch_result_2 =
            conn.fetch_one(&format!(r"SELECT * FROM {} WHERE name = 'two'", table_name), &[], mapper);
        assert!(fetch_result_2.is_err());

        assert!(conn.execute(&format!(r"DROP TABLE {}", table_name), &[]).is_ok());
    });
}

#[test]
fn should_execute_and_fetch_option() {
    SINGLETON.get(|(pool, _)| {
        let c3p0: C3p0Impl = pool.clone();

        let conn = c3p0.connection().unwrap();

        let table_name = format!("TEST_TABLE_{}", rand_string(8));
        assert!(conn
            .execute(
                &format!(r"CREATE TABLE {} (
                             name varchar(255)
                          )", table_name),
                &[]
            )
            .is_ok());

        #[cfg(feature = "pg")]
        let insert = &format!(r"INSERT INTO {} (name) VALUES ($1)", table_name);
        #[cfg(any(feature = "mysql", feature = "sqlite"))]
        let insert = &format!(r"INSERT INTO {} (name) VALUES (?)", table_name);

        assert_eq!(1, conn.execute(insert, &[&"one"]).unwrap());

        #[cfg(feature = "pg")]
        let mapper = |row: &Row| Ok(row.get(0));
        #[cfg(feature = "mysql")]
        let mapper = |row: &Row| {
            Ok(row
                .get(0)
                .ok_or_else(|| c3p0_json::C3p0Error::ResultNotFoundError)?)
        };
        #[cfg(feature = "sqlite")]
        let mapper = |row: &Row| Ok(row.get(0)?);

        let fetch_result =
            conn.fetch_one_option(&format!(r"SELECT * FROM {} WHERE name = 'one'", table_name), &[], mapper);
        assert!(fetch_result.is_ok());
        assert_eq!(Some("one".to_owned()), fetch_result.unwrap());

        let fetch_result =
            conn.fetch_one_option(&format!(r"SELECT * FROM {} WHERE name = 'two'", table_name), &[], mapper);
        assert!(fetch_result.is_ok());
        assert_eq!(None, fetch_result.unwrap());

        assert!(conn.execute(&format!(r"DROP TABLE {}", table_name), &[]).is_ok());
    });
}

#[test]
fn should_batch_execute() {
    SINGLETON.get(|(pool, _)| {
        let c3p0: C3p0Impl = pool.clone();
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
fn should_fetch_values() {
    SINGLETON.get(|(pool, _)| {
        let c3p0: C3p0Impl = pool.clone();

        c3p0.transaction(|conn| {
            assert!(conn
                .batch_execute("CREATE TABLE TEST_TABLE ( name varchar(255) )")
                .is_ok());

            let all_string = conn.fetch_all_values::<String>("SELECT * FROM TEST_TABLE", &[]);
            assert!(all_string.is_ok());
            assert!(all_string.unwrap().is_empty());

            let one_string = conn.fetch_one_value::<String>("SELECT * FROM TEST_TABLE", &[]);
            assert!(one_string.is_err());

            let one_i64 = conn.fetch_one_value::<i64>("SELECT * FROM TEST_TABLE", &[]);
            assert!(one_i64.is_err());

            conn.batch_execute(
                r"INSERT INTO TEST_TABLE (name) VALUES ('one');
                                    INSERT INTO TEST_TABLE (name) VALUES ('two');
                                    INSERT INTO TEST_TABLE (name) VALUES ('three')",
            )
            .unwrap();

            let all_string =
                conn.fetch_all_values::<String>("SELECT name FROM TEST_TABLE order by name", &[]);
            assert!(all_string.is_ok());
            assert_eq!(
                vec!["one".to_owned(), "three".to_owned(), "two".to_owned(),],
                all_string.unwrap()
            );

            let all_i64 = conn.fetch_all_values::<i64>("SELECT * FROM TEST_TABLE", &[]);
            assert!(all_i64.is_err());

            let one_string =
                conn.fetch_one_value::<String>("SELECT name FROM TEST_TABLE order by name", &[]);
            assert!(one_string.is_ok());
            assert_eq!("one".to_owned(), one_string.unwrap());

            let one_i64 = conn.fetch_one_value::<i64>("SELECT * FROM TEST_TABLE", &[]);
            assert!(one_i64.is_err());

            assert!(conn.batch_execute("DROP TABLE TEST_TABLE").is_ok());

            Ok(())
        })
        .unwrap();
    });
}