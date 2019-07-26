use crate::migration::{Migration, Migrations};
use c3p0_common::error::C3p0Error;
use std::convert::TryFrom;
use std::fs::read_to_string;
use std::path::Path;
use walkdir::WalkDir;

impl TryFrom<String> for Migrations {
    type Error = C3p0Error;

    fn try_from(path: String) -> Result<Self, Self::Error> {
        from_fs(&path)
    }
}

impl<'a> TryFrom<&'a str> for Migrations {
    type Error = C3p0Error;

    fn try_from(path: &'a str) -> Result<Self, Self::Error> {
        from_fs(path)
    }
}

pub fn from_fs<P: AsRef<Path>>(path_ref: P) -> Result<Migrations, C3p0Error> {
    let mut migrations = vec![];

    let path = path_ref.as_ref();

    for entry in WalkDir::new(path)
        .min_depth(1)
        .max_depth(1)
        .sort_by(|a, b| a.file_name().cmp(b.file_name()))
        .into_iter()
        .filter_entry(|e| e.path().is_dir())
        .filter_map(std::result::Result::ok)
    {
        //println!("analise: {}", entry.path().display());

        let id = entry
            .path()
            .file_name()
            .and_then(std::ffi::OsStr::to_str)
            .ok_or_else(|| C3p0Error::FileSystemError {
                message: format!("Cannot get filename of [{}]", entry.path().display()),
            })?;

        let up = entry.path().join("up.sql");
        let up_script = read_to_string(up.as_path()).map_err(|err| C3p0Error::FileSystemError {
            message: format!("Error reading file [{}]. Err: [{}]", up.display(), err),
        })?;

        let down = entry.path().join("down.sql");
        let down_script =
            read_to_string(down.as_path()).map_err(|err| C3p0Error::FileSystemError {
                message: format!("Error reading file [{}]. Err: [{}]", down.display(), err),
            })?;

        migrations.push(Migration {
            id: id.to_owned(),
            down: down_script,
            up: up_script,
        })
    }

    migrations.sort_by(|first, second| first.id.cmp(&second.id));

    Ok(Migrations { migrations })
}

#[cfg(test)]
mod test {

    use super::*;
    use std::convert::TryInto;

    #[test]
    fn should_read_migrations_from_fs() {
        let migrations: Migrations = "./tests/migrations_00".try_into().unwrap();

        assert_eq!(2, migrations.migrations.len());

        assert_eq!("00010_create_test_data", migrations.migrations[0].id);
        assert!(migrations.migrations[0]
            .up
            .contains("create table TEST_TABLE"));
        assert!(migrations.migrations[0]
            .down
            .contains("DROP TABLE TEST_TABLE;"));

        assert_eq!("00011_insert_test_data", migrations.migrations[1].id);
        assert!(migrations.migrations[1]
            .up
            .contains("INSERT INTO TEST_TABLE (id, name) VALUES ('one', 'one');"));
        assert!(migrations.migrations[1]
            .down
            .contains("delete from TEST_TABLE;"));
    }

    #[test]
    fn should_return_error_if_missing_down_sql() {
        let migrations: Result<Migrations, _> = "./tests/migrations_01".try_into();
        assert!(migrations.is_err());

        match migrations {
            Err(C3p0Error::FileSystemError { message }) => {
                assert!(message.contains(
                    "Error reading file [./tests/migrations_01/00010_create_test_data/down.sql]"
                ));
            }
            _ => assert!(false),
        };
    }

    #[test]
    fn should_return_error_if_missing_up_sql() {
        let migrations: Result<Migrations, _> = "./tests/migrations_02".try_into();
        assert!(migrations.is_err());

        match migrations {
            Err(C3p0Error::FileSystemError { message }) => {
                assert!(message.contains(
                    "Error reading file [./tests/migrations_02/00010_create_test_data/up.sql]"
                ));
            }
            _ => assert!(false),
        };
    }

    #[test]
    fn should_ignore_inner_folders() {
        let migrations: Migrations = "./tests/migrations_03".try_into().unwrap();

        assert_eq!(1, migrations.migrations.len());

        assert_eq!("00010_create_test_data", migrations.migrations[0].id);
        assert!(migrations.migrations[0]
            .up
            .contains("create table TEST_TABLE"));
        assert!(migrations.migrations[0]
            .down
            .contains("DROP TABLE TEST_TABLE;"));
    }

}
