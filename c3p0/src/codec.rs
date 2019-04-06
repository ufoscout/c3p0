use crate::error::C3p0Error;
use serde_json::Value;

#[derive(Clone)]
pub struct Codec<DATA>
where
    DATA: serde::ser::Serialize + serde::de::DeserializeOwned,
{
    pub from_value: fn(value: Value) -> Result<DATA, C3p0Error>,
    pub to_value: fn(data: &DATA) -> Result<Value, C3p0Error>,
}

impl<DATA> Default for Codec<DATA>
where
    DATA: serde::ser::Serialize + serde::de::DeserializeOwned,
{
    fn default() -> Self {
        Codec {
            from_value: |value| serde_json::from_value::<DATA>(value).map_err(C3p0Error::from),
            to_value: |data| serde_json::to_value(data).map_err(C3p0Error::from),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_derive::{Deserialize, Serialize};
    use serde_json::Value;
    use std::borrow::Cow;

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
    fn check_json_type_format_with_cow() {
        let v2 = UserVersion2 {
            username: "user_v1_name".to_owned(),
            email: "user_v1_email@test.com".to_owned(),
            age: 123,
        };
        let version2 = Versioning2::V2(Cow::Borrowed(&v2));

        let json = serde_json::to_string(&version2).unwrap();

        println!("json format: \n{}", json)
    }

    #[test]
    fn codec_should_code_and_decode() {
        let v2 = UserVersion2 {
            username: "user_v1_name".to_owned(),
            email: "user_v1_email@test.com".to_owned(),
            age: 123,
        };

        let codec = Codec {
            to_value: |data| Versioning2::to_value(data),
            from_value: |value| Versioning2::from_value(value),
        };

        let v2_value = (codec.to_value)(&v2).unwrap();
        let v2_decode = (codec.from_value)(v2_value).unwrap();
        assert_eq!(v2.username, v2_decode.username);
        assert_eq!(v2.email, v2_decode.email);
        assert_eq!(v2.age, v2_decode.age);
    }

    #[test]
    fn codec_should_upgrade_between_versions() {
        let v1 = UserVersion1 {
            username: "user_v1_name".to_owned(),
            email: "user_v1_email@test.com".to_owned(),
        };

        let v1_value = Versioning1::to_value(&v1).unwrap();

        let codec = Codec {
            to_value: |data| Versioning2::to_value(data),
            from_value: |value| Versioning2::from_value(value),
        };

        let v2 = (codec.from_value)(v1_value).unwrap();
        assert_eq!(v1.username, v2.username);
        assert_eq!(v1.email, v2.email);
        assert_eq!(18, v2.age);
    }
}
