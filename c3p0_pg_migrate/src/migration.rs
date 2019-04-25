use walkdir::WalkDir;
use std::convert::TryFrom;
use std::fs::read_to_string;
use crate::error::C3p0MigrateError;

#[derive(Clone, Debug, PartialEq)]
pub struct Migration {
    pub id: String,
    pub up: String,
    pub down: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Migrations {
    pub migrations: Vec<Migration>,
}

impl From<Vec<Migration>> for Migrations {
    fn from(migrations: Vec<Migration>) -> Self {
        Migrations{
            migrations
        }
    }
}

impl TryFrom<&str> for Migrations {
    type Error = C3p0MigrateError;

    fn try_from(path: &str) -> Result<Self, Self::Error> {

        let mut migrations = vec![];

        for entry in WalkDir::new(path)
            .max_depth(1)
            .sort_by(|a, b| a.path().cmp(b.path()))
            .into_iter()
            .filter_entry(|e| e.path().is_dir())
            .filter_map(|e| e.ok()) {

            //println!("analise: {}", entry.path().display());

            let direct_child = if let Some(parent) = entry.path().parent() {
                //println!("parent: {}", parent.display());
                same_file::is_same_file(parent, path).unwrap_or_else(|_e| false)
            } else {
                false
            };

            println!("is children: {}", direct_child);

            if direct_child {

                let id = entry.path().file_name().and_then(|os_name| os_name.to_str())
                    .ok_or_else(|| C3p0MigrateError::FileSystemError {message: format!("Cannot get filename of [{}]", entry.path().display())})?;

                let up = entry.path().join("up.sql");
                let up_script = read_to_string(up.as_path())
                    .map_err(|err| C3p0MigrateError::FileSystemError {message: format!("Error reading file [{}]. Err: [{}]", up.display(), err)})?;

                let down = entry.path().join("down.sql");
                let down_script = read_to_string(down.as_path())
                    .map_err(|err| C3p0MigrateError::FileSystemError {message: format!("Error reading file [{}]. Err: [{}]", up.display(), err)})?;

                migrations.push(Migration{
                    id: id.to_owned(),
                    down: down_script,
                    up: up_script
                })
            }

        };

        Ok(Migrations{
            migrations
        })
    }
}



#[cfg(test)]
mod test {

    use super::*;
    use std::convert::TryInto;

    #[test]
    fn should_read_migrations_from_fs() {
        let migrations: Migrations = "./tests/migrations".try_into().unwrap();

        assert_eq!(2, migrations.migrations.len())

    }

}
