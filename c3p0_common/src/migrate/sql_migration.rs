use crate::migrate::migration::Migration;

pub fn to_sql_migrations(migrations: Vec<Migration>) -> Vec<SqlMigration> {
    migrations.into_iter().map(SqlMigration::new).collect()
}

#[derive(Clone, Debug, PartialEq)]
pub struct SqlMigration {
    pub id: String,
    pub up: SqlScript,
    pub down: SqlScript,
}

impl SqlMigration {
    pub fn new(migration: Migration) -> SqlMigration {
        SqlMigration {
            id: migration.id,
            up: SqlScript::new(migration.up),
            down: SqlScript::new(migration.down),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SqlScript {
    pub sql: String,
    pub md5: String,
}

impl SqlScript {
    pub fn new<S: Into<String>>(sql: S) -> SqlScript {
        let sql = sql.into();
        let md5 = crate::migrate::md5::calculate_md5(&sql);
        SqlScript { sql, md5 }
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn should_create_migrations_with_md5() {
        let source_sqls = vec![
            Migration {
                id: "first".to_owned(),
                up: "insert into table1".to_owned(),
                down: "delete from table1".to_owned(),
            },
            Migration {
                id: "second".to_owned(),
                up: "insert into table2".to_owned(),
                down: "delete from table2".to_owned(),
            },
        ];

        let migrations = to_sql_migrations(source_sqls.clone());

        assert_eq!(2, migrations.len());

        assert_eq!(
            &SqlMigration {
                id: "first".to_owned(),
                up: SqlScript {
                    sql: "insert into table1".to_owned(),
                    md5: "7f1145b3d3b6654e3388610423abb12f".to_owned(),
                },
                down: SqlScript {
                    sql: "delete from table1".to_owned(),
                    md5: "7e8ab3d9327f4f1a80e2b9de1acc35c0".to_owned(),
                }
            },
            migrations.first().unwrap()
        );

        assert_eq!(
            &SqlMigration {
                id: "second".to_owned(),
                up: SqlScript {
                    sql: "insert into table2".to_owned(),
                    md5: "7233bb544a3a701c33590a8c72d74e22".to_owned(),
                },
                down: SqlScript {
                    sql: "delete from table2".to_owned(),
                    md5: "116ee10121cdb2cc04c3c523c51af1d3".to_owned(),
                }
            },
            migrations.get(1).unwrap()
        );
    }
}
