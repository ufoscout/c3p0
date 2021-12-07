use crate::error::C3p0Error;
use crate::migrate::migration::{Migration, Migrations};
use include_dir::Dir;
use std::convert::TryFrom;

impl<'a> TryFrom<&'a Dir<'a>> for Migrations {
    type Error = C3p0Error;

    fn try_from(path: &'a Dir) -> Result<Self, Self::Error> {
        from_embed(path)
    }
}

pub fn from_embed(dir: &Dir) -> Result<Migrations, C3p0Error> {
    let mut migrations = vec![];

    for entry in dir.dirs() {
        // println!("Found {}", entry.path().display());

        let id = entry
            .path()
            .file_name()
            .and_then(std::ffi::OsStr::to_str)
            .ok_or_else(|| C3p0Error::IoError {
                message: format!("Cannot get filename of [{}]", entry.path().display()),
            })?;

        // for files in entry.files() {
        // println!("Found files: {}", files.path().display());
        // }

        let up_filename = format!("{}/up.sql", entry.path().display());
        let up_script = entry
            .get_file(&up_filename)
            .and_then(|file| file.contents_utf8())
            .ok_or_else(|| C3p0Error::IoError {
                message: format!("Error reading file [{}].", up_filename),
            })?;

        let down_filename = format!("{}/down.sql", entry.path().display());
        let down_script = entry
            .get_file(&down_filename)
            .and_then(|file| file.contents_utf8())
            .ok_or_else(|| C3p0Error::IoError {
                message: format!("Error reading file [{}].", down_filename),
            })?;

        migrations.push(Migration {
            id: id.to_owned(),
            down: down_script.to_owned(),
            up: up_script.to_owned(),
        })
    }

    migrations.sort_by(|first, second| first.id.cmp(&second.id));

    Ok(Migrations { migrations })
}

#[cfg(test)]
mod test {

    use super::*;
    use include_dir::*;
    use std::convert::TryInto;

    const MIGRATIONS_00: Dir = include_dir!("$CARGO_MANIFEST_DIR/tests/migrations_00");
    const MIGRATIONS_01: Dir = include_dir!("$CARGO_MANIFEST_DIR/tests/migrations_01");
    const MIGRATIONS_02: Dir = include_dir!("$CARGO_MANIFEST_DIR/tests/migrations_02");
    const MIGRATIONS_03: Dir = include_dir!("$CARGO_MANIFEST_DIR/tests/migrations_03");

    #[test]
    fn should_read_migrations_from_embed() {
        let migrations: Migrations = (&MIGRATIONS_00).try_into().unwrap();

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
        let migrations: Result<Migrations, _> = (&MIGRATIONS_01).try_into();
        assert!(migrations.is_err());

        match migrations {
            Err(C3p0Error::IoError { message }) => {
                assert!(message.contains("Error reading file [00010_create_test_data/down.sql]"));
            }
            _ => assert!(false),
        };
    }

    #[test]
    fn should_return_error_if_missing_up_sql() {
        let migrations: Result<Migrations, _> = (&MIGRATIONS_02).try_into();
        assert!(migrations.is_err());

        match migrations {
            Err(C3p0Error::IoError { message }) => {
                assert!(message.contains("Error reading file [00010_create_test_data/up.sql]"));
            }
            _ => assert!(false),
        };
    }

    #[test]
    fn should_ignore_inner_folders() {
        let migrations: Migrations = (&MIGRATIONS_03).try_into().unwrap();

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
