pub mod embed;
pub mod fs;

pub use embed::*;
pub use fs::*;

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
