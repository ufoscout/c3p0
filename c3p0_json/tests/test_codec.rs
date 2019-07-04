use c3p0_json::*;
use serde_derive::{Deserialize, Serialize};
use serde_json::Value;
use std::borrow::Cow;

#[cfg(feature = "pg")]
mod shared_pg;
#[cfg(feature = "pg")]
use crate::shared_pg::*;

#[cfg(feature = "mysql")]
mod shared_mysql;
#[cfg(feature = "mysql")]
use crate::shared_mysql::*;

#[cfg(feature = "sqlite")]
mod shared_sqlite;
#[cfg(feature = "sqlite")]
use crate::shared_sqlite::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct UserVersion1 {
    pub username: String,
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct UserVersion2 {
    pub username: String,
    pub email: String,
    pub age: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "@json_tag")]
enum Versioning1<'a> {
    V1(Cow<'a, UserVersion1>),
}

struct UserVersionCoded1 {}

impl JsonCodec<UserVersion1> for UserVersionCoded1 {
    fn from_value(&self, value: Value) -> Result<UserVersion1, C3p0Error> {
        let versioning = serde_json::from_value(value)?;
        let user = match versioning {
            Versioning1::V1(user_v1) => user_v1.into_owned(),
        };
        Ok(user)
    }

    fn to_value(&self, data: &UserVersion1) -> Result<Value, C3p0Error> {
        serde_json::to_value(Versioning1::V1(Cow::Borrowed(data))).map_err(C3p0Error::from)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "@json_tag")]
enum Versioning2<'a> {
    V1(UserVersion1),
    V2(Cow<'a, UserVersion2>),
}

struct UserVersionCoded2 {}

impl JsonCodec<UserVersion2> for UserVersionCoded2 {
    fn from_value(&self, value: Value) -> Result<UserVersion2, C3p0Error> {
        let versioning = serde_json::from_value(value)?;
        let user = match versioning {
            Versioning2::V1(user_v1) => UserVersion2 {
                username: user_v1.username,
                email: user_v1.email,
                age: 18,
            },
            Versioning2::V2(user_v2) => user_v2.into_owned(),
        };
        Ok(user)
    }

    fn to_value(&self, data: &UserVersion2) -> Result<Value, C3p0Error> {
        serde_json::to_value(Versioning2::V2(Cow::Borrowed(data))).map_err(C3p0Error::from)
    }
}

#[test]
fn should_upgrade_structs_on_load() {
    SINGLETON.get(|(pool, _)| {
        let conn = pool.connection().unwrap();
        let table_name = "USER_TABLE";

        let jpo_v1 = C3p0JsonBuilderImpl::new(table_name)
            .with_codec(UserVersionCoded1 {})
            .build();

        let jpo_v2 = C3p0JsonBuilderImpl::new(table_name)
            .with_codec(UserVersionCoded2 {})
            .build();

        let new_user_v1 = NewModel::new(UserVersion1 {
            username: "user_v1_name".to_owned(),
            email: "user_v1_email@test.com".to_owned(),
        });

        assert!(jpo_v1.create_table_if_not_exists(&conn).is_ok());
        assert!(jpo_v1.delete_all(&conn).is_ok());

        let user_v1 = jpo_v1.save(&conn, new_user_v1.clone()).unwrap();
        println!("user id is {}", user_v1.id);
        println!("total users: {}", jpo_v1.count_all(&conn).unwrap());
        println!(
            "select all users len: {}",
            jpo_v1.find_all(&conn).unwrap().len()
        );

        let user_v2_found = jpo_v2.find_by_id(&conn, &user_v1.id).unwrap();
        assert!(user_v2_found.is_some());

        let user_v2_found = user_v2_found.unwrap();
        assert_eq!(user_v1.id, user_v2_found.id);
        assert_eq!(user_v1.version, user_v2_found.version);
        assert_eq!(user_v1.data.username, user_v2_found.data.username);
        assert_eq!(user_v1.data.email, user_v2_found.data.email);
        assert_eq!(18, user_v2_found.data.age);
    });
}
