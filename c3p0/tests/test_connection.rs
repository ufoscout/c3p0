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
fn should_execute_sql() {
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

        assert_eq!(
            1,
            conn.execute(insert, &[&"one"])
                .unwrap()
        );

        assert!(conn.execute(r"DROP TABLE TEST_TABLE", &[]).is_ok());
    });
}
