use c3p0::codec::Codec;
use c3p0::error::C3p0Error;
use c3p0_mysql::{C3p0, C3p0Repository, Config, ConfigBuilder, NewModel};
use serde_derive::{Deserialize, Serialize};
use serde_json::Value;
use std::borrow::Cow;

mod shared;

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
#[serde(tag = "json_version")]
enum Versioning1<'a> {
    V1(Cow<'a, UserVersion1>),
}

impl<'a> Versioning1<'a> {
    fn from_value(value: Value) -> Result<UserVersion1, C3p0Error> {
        let versioning = serde_json::from_value(value)?;
        let user = match versioning {
            Versioning1::V1(user_v1) => user_v1.into_owned(),
        };
        Ok(user)
    }

    fn to_value(data: &'a UserVersion1) -> Result<Value, C3p0Error> {
        serde_json::to_value(Versioning1::V1(Cow::Borrowed(data))).map_err(C3p0Error::from)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "json_version")]
enum Versioning2<'a> {
    V1(UserVersion1),
    V2(Cow<'a, UserVersion2>),
}

impl<'a> Versioning2<'a> {
    fn from_value(value: Value) -> Result<UserVersion2, C3p0Error> {
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

    fn to_value(data: &'a UserVersion2) -> Result<Value, C3p0Error> {
        serde_json::to_value(Versioning2::V2(Cow::Borrowed(data))).map_err(C3p0Error::from)
    }
}

#[test]
fn should_create_and_drop_table() {
    shared::SINGLETON.get(|(pool, _)| {
        let mut conn = pool.get().unwrap();
        let table_name = "USER_TABLE";

        let conf_v1: Config<UserVersion1> = ConfigBuilder::new(table_name)
            .with_codec(Codec {
                to_value: |data| Versioning1::to_value(data),
                from_value: |value| Versioning1::from_value(value),
            })
            .build();
        let jpo_v1 = C3p0Repository::build(conf_v1);

        let conf_v2: Config<UserVersion2> = ConfigBuilder::new(table_name)
            .with_codec(Codec {
                to_value: |data| Versioning2::to_value(data),
                from_value: |value| Versioning2::from_value(value),
            })
            .build();
        let jpo_v2 = C3p0Repository::build(conf_v2);

        let new_user_v1 = NewModel::new(UserVersion1 {
            username: "user_v1_name".to_owned(),
            email: "user_v1_email@test.com".to_owned(),
        });

        assert!(jpo_v1.create_table_if_not_exists(&mut conn).is_ok());
        assert!(jpo_v1.delete_all(&mut conn).is_ok());

        let user_v1 = jpo_v1.save(&mut conn, new_user_v1.clone()).unwrap();

        let user_v2_found = jpo_v2.find_by_id(&mut conn, &user_v1.id).unwrap();
        assert!(user_v2_found.is_some());

        let user_v2_found = user_v2_found.unwrap();
        assert_eq!(user_v1.id, user_v2_found.id);
        assert_eq!(user_v1.version, user_v2_found.version);
        assert_eq!(user_v1.data.username, user_v2_found.data.username);
        assert_eq!(user_v1.data.email, user_v2_found.data.email);
        assert_eq!(18, user_v2_found.data.age);
    });
}
