pub mod embed;
pub mod fs;

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
        Migrations { migrations }
    }
}
