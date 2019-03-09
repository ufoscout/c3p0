#[derive(Clone, Debug, PartialEq)]
pub struct Migrations {
    pub migrations: Vec<SqlMigration>
}

impl Migrations {
    pub fn new(migrations: Vec<Migration>) -> Migrations {
        let migrations = migrations.into_iter().map(|migration| {
            SqlMigration::new(migration)
        }).collect();
        Migrations{
            migrations
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Migration {
    pub up: String,
    pub down: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SqlMigration {
    pub up: SqlScript,
    pub down: SqlScript,
}

impl SqlMigration {
    pub fn new(migration: Migration) -> SqlMigration {
        SqlMigration{
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

impl SqlScript{
    pub fn new<S: Into<String>>(sql: S) -> SqlScript {
        let sql = sql.into();
        let md5 = crate::md5::calculate_md5(&sql);
        SqlScript{
            sql,
            md5
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn should_create_migrations_with_md5() {
        let source_sqls = vec!(
            Migration{
                up: "insert into table1".to_owned(),
                down: "delete from table1".to_owned(),
            },
            Migration{
                up: "insert into table2".to_owned(),
                down: "delete from table2".to_owned(),
            }
        );

        let migrations = Migrations::new(source_sqls.clone());

        assert_eq!(2, migrations.migrations.len());

        assert_eq!(&SqlMigration{
            up: SqlScript{
                sql: "insert into table1".to_owned(),
                md5: "7f1145b3d3b6654e3388610423abb12f".to_owned(),
            },
            down: SqlScript{
                sql: "delete from table1".to_owned(),
                md5: "7e8ab3d9327f4f1a80e2b9de1acc35c0".to_owned(),
            }
        }, migrations.migrations.get(0).unwrap());

        assert_eq!(&SqlMigration{
            up: SqlScript{
                sql: "insert into table2".to_owned(),
                md5: "7233bb544a3a701c33590a8c72d74e22".to_owned(),
            },
            down: SqlScript{
                sql: "delete from table2".to_owned(),
                md5: "116ee10121cdb2cc04c3c523c51af1d3".to_owned(),
            }
        }, migrations.migrations.get(1).unwrap());

    }

}