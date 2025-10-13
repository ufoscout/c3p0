use serde::{Deserialize, Serialize};

use crate::record::*;

pub mod error;
pub mod pool;
pub mod record;
pub mod time;

#[cfg(feature = "postgres")]
pub mod postgres;
#[cfg(feature = "mysql")]
pub mod mysql;
#[cfg(feature = "sqlite")]
pub mod sqlite;

pub type UserRecord = Record<User>;
pub type UserNewRecord = NewRecord<User>;

#[derive(Serialize, Deserialize)]
pub struct User {
    pub name: String,
    pub age: i32,
}

impl Data for User {
    const TABLE_NAME: &'static str = "users";
    type ID = u32;
    // type CODEC = User;
    type CODEC = VersionedUser;
}

#[derive(Serialize, Deserialize)]
pub enum VersionedUser {
    V1(String),
    V2(User),
}

impl Codec<User> for VersionedUser {
    fn encode(data: User) -> Self {
        VersionedUser::V2(data)
    }

    fn decode(data: Self) -> User {
        match data {
            VersionedUser::V1(name) => User { name, age: 0 },
            VersionedUser::V2(user) => user,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn print_table_name() {
        let user = UserRecord {
            id: 1,
            data: User { name: "John".to_string(), age: 30 }
        };

        assert_eq!(User::TABLE_NAME, "users");
        assert_eq!(user.select(), "select * from users where id = $1");

        let user = UserNewRecord {
            data: User { name: "John".to_string(), age: 30 }
        };

        assert_eq!(user.insert(), "insert into users");
    }
}